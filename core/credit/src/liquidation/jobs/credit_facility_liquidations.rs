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
    CoreCreditEvent, CreditFacilityId, LedgerOmnibusAccountIds, LiquidationRepo, NewLiquidation,
    collateral::CollateralRepo,
    credit_facility::CreditFacilityRepo,
    liquidation::{
        LiquidationId, NewLiquidationBuilder,
        error::LiquidationError,
        jobs::{
            liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner},
            partial_liquidation::{PartialLiquidationJobConfig, PartialLiquidationJobSpawner},
        },
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
    liquidation_repo: Arc<LiquidationRepo<E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
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
        liquidation_repo: Arc<LiquidationRepo<E>>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        collateral_repo: Arc<CollateralRepo<E>>,
        proceeds_omnibus_account_ids: &LedgerOmnibusAccountIds,
        partial_liquidation_job_spawner: PartialLiquidationJobSpawner<E>,
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            liquidation_repo,
            credit_facility_repo,
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
            liquidation_repo: self.liquidation_repo.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
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
    liquidation_repo: Arc<LiquidationRepo<E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
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
                                .liquidation_repo
                                .begin_op_with_clock(current_job.clock())
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
                liquidation_id,
                credit_facility_id,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
                collateral_account_id,
                collateral_in_liquidation_account_id,
                liquidated_collateral_account_id,
                proceeds_from_liquidation_account_id,
                payment_holding_account_id,
                uncovered_outstanding_account_id,
                ..
            },
        ) = message.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            if let Some(liquidation_id) = self
                .create_if_not_exist_for_facility_in_op(
                    db,
                    *credit_facility_id,
                    NewLiquidation::builder()
                        .id(*liquidation_id)
                        .credit_facility_id(*credit_facility_id)
                        .facility_proceeds_from_liquidation_account_id(
                            *proceeds_from_liquidation_account_id,
                        )
                        .facility_payment_holding_account_id(*payment_holding_account_id)
                        .collateral_account_id(*collateral_account_id)
                        .facility_uncovered_outstanding_account_id(
                            *uncovered_outstanding_account_id,
                        )
                        .collateral_in_liquidation_account_id(*collateral_in_liquidation_account_id)
                        .liquidated_collateral_account_id(*liquidated_collateral_account_id)
                        .trigger_price(*trigger_price)
                        .initially_expected_to_receive(*initially_expected_to_receive)
                        .initially_estimated_to_liquidate(*initially_estimated_to_liquidate),
                )
                .await?
            {
                self.enter_collateral_liquidation(db, *credit_facility_id, liquidation_id)
                    .await?;

                self.partial_liquidation_job_spawner
                    .spawn_in_op(
                        db,
                        JobId::new(),
                        PartialLiquidationJobConfig::<E> {
                            liquidation_id,
                            credit_facility_id: *credit_facility_id,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .await?;
            }
        }
        Ok(())
    }

    async fn enter_collateral_liquidation(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        liquidation_id: LiquidationId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let credit_facility = self
            .credit_facility_repo
            .find_by_id_in_op(db, credit_facility_id)
            .await?;
        let mut collateral = self
            .collateral_repo
            .find_by_id_in_op(&mut *db, credit_facility.collateral_id)
            .await?;

        if collateral.enter_liquidation(liquidation_id)?.did_execute() {
            self.collateral_repo
                .update_in_op(&mut *db, &mut collateral)
                .await?;
        }

        Ok(())
    }

    #[instrument(
        name = "credit.liquidation.create_if_not_exist_for_facility_in_op",
        skip(self, db, new_liquidation),
        fields(existing_liquidation_found),
        err
    )]
    pub async fn create_if_not_exist_for_facility_in_op(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        new_liquidation: &mut NewLiquidationBuilder,
    ) -> Result<Option<LiquidationId>, LiquidationError> {
        let existing_liquidation = self
            .liquidation_repo
            .maybe_find_active_liquidation_for_credit_facility_id_in_op(
                &mut *db,
                credit_facility_id,
            )
            .await?;

        tracing::Span::current()
            .record("existing_liquidation_found", existing_liquidation.is_some());

        if existing_liquidation.is_some() {
            Ok(None)
        } else {
            let liquidation = self
                .liquidation_repo
                .create_in_op(
                    db,
                    new_liquidation
                        .liquidation_proceeds_omnibus_account_id(
                            self.proceeds_omnibus_account_ids.account_id,
                        )
                        .build()
                        .expect("Could not build new liquidation"),
                )
                .await?;

            self.partial_liquidation_job_spawner
                .spawn_in_op(
                    db,
                    JobId::new(),
                    PartialLiquidationJobConfig::<E> {
                        liquidation_id: liquidation.id,
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
                        liquidation_id: liquidation.id,
                        credit_facility_id,
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
            Ok(Some(liquidation.id))
        }
    }
}
