use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use std::sync::Arc;

use core_custody::CoreCustodyEvent;
use es_entity::DbOp;
use governance::GovernanceEvent;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CollateralId, CoreCreditEvent, CreditFacilityId, LedgerOmnibusAccountIds,
    collateral::{CollateralRepo, error::CollateralError},
    liquidation::jobs::{
        liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner},
        partial_liquidation::{PartialLiquidationJobConfig, PartialLiquidationJobSpawner},
    },
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityLiquidationsJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityLiquidationsJobConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}

pub struct CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    collateral_repo: Arc<CollateralRepo<E>>,
    proceeds_omnibus_account_ids: LedgerOmnibusAccountIds,
    partial_liquidation_job_spawner: PartialLiquidationJobSpawner<E>,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

impl<E> CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        collateral_repo: Arc<CollateralRepo<E>>,
        proceeds_omnibus_account_ids: &LedgerOmnibusAccountIds,
        partial_liquidation_job_spawner: PartialLiquidationJobSpawner<E>,
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            collateral_repo,
            proceeds_omnibus_account_ids: proceeds_omnibus_account_ids.clone(),
            partial_liquidation_job_spawner,
            liquidation_payment_job_spawner,
        }
    }
}

const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");
impl<E> JobInitializer for CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = CreditFacilityLiquidationsJobConfig<E>;
    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_LIQUIDATIONS_JOB
    }

    fn init(
        &self,
        _job: &job::Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityLiquidationsJobRunner::<E> {
            outbox: self.outbox.clone(),
            collateral_repo: self.collateral_repo.clone(),
            proceeds_omnibus_account_ids: self.proceeds_omnibus_account_ids.clone(),
            partial_liquidation_job_spawner: self.partial_liquidation_job_spawner.clone(),
            liquidation_payment_job_spawner: self.liquidation_payment_job_spawner.clone(),
        }))
    }
}

pub struct CreditFacilityLiquidationsJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    collateral_repo: Arc<CollateralRepo<E>>,
    proceeds_omnibus_account_ids: LedgerOmnibusAccountIds,
    partial_liquidation_job_spawner: PartialLiquidationJobSpawner<E>,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

#[async_trait]
impl<E> JobRunner for CreditFacilityLiquidationsJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityLiquidationsJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %CREDIT_FACILITY_LIQUIDATIONS_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {

                            let mut db = self
                                .collateral_repo
                                .begin_op()
                                .await?;
                            self.process_message(&mut db, message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;
                            db.commit().await?;
                        }
                        None => return Ok(JobCompletion::RescheduleNow)
                    }
                }
            }
        }
    }
}

impl<E> CreditFacilityLiquidationsJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "outbox.core_credit.credit_facility_liquidations.process_message", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(
            event @ CoreCreditEvent::PartialLiquidationInitiated {
                credit_facility_id,
                collateral_id,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
                ..
            },
        ) = message.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            self.initiate_liquidation_via_collateral_in_op(
                db,
                *credit_facility_id,
                *collateral_id,
                *trigger_price,
                *initially_expected_to_receive,
                *initially_estimated_to_liquidate,
            )
            .await?;
        }
        Ok(())
    }

    #[instrument(
        name = "credit.liquidation.initiate_liquidation_via_collateral_in_op",
        skip(self, db),
        fields(existing_liquidation_found),
        err
    )]
    pub async fn initiate_liquidation_via_collateral_in_op(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        collateral_id: CollateralId,
        trigger_price: crate::PriceOfOneBTC,
        initially_expected_to_receive: crate::UsdCents,
        initially_estimated_to_liquidate: crate::Satoshis,
    ) -> Result<(), CollateralError> {
        let mut collateral = self.collateral_repo.find_by_id(collateral_id).await?;

        let has_active = collateral.has_active_liquidation();
        tracing::Span::current().record("existing_liquidation_found", has_active);

        if has_active {
            return Ok(());
        }

        let liquidation_id = match collateral.initiate_liquidation(
            self.proceeds_omnibus_account_ids.account_id,
            trigger_price,
            initially_expected_to_receive,
            initially_estimated_to_liquidate,
        )? {
            es_entity::Idempotent::Executed(id) => id,
            es_entity::Idempotent::AlreadyApplied => {
                // This shouldn't happen since we checked has_active_liquidation above
                return Ok(());
            }
        };

        self.collateral_repo.update_in_op(db, &mut collateral).await?;

        self.partial_liquidation_job_spawner
            .spawn_in_op(
                db,
                JobId::new(),
                PartialLiquidationJobConfig::<E> {
                    liquidation_id,
                    collateral_id,
                    credit_facility_id,
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;
        self.liquidation_payment_job_spawner
            .spawn_in_op(
                db,
                JobId::new(),
                LiquidationPaymentJobConfig::<E> {
                    liquidation_id,
                    credit_facility_id,
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        Ok(())
    }
}
