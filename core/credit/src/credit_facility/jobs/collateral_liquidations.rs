use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::CoreCustodyEvent;
use governance::GovernanceEvent;
use job::*;
use money::{Satoshis, UsdCents};
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CalaAccountId, CoreCreditEvent,
    collateral::{Collaterals, LiquidationInitiated, error::CollateralError},
    primitives::*,
};

use super::liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner};

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityLiquidationsJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityLiquidationsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for CreditFacilityLiquidationsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for CreditFacilityLiquidationsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct CreditFacilityLiquidationsInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    collaterals: Arc<Collaterals<Perms, E>>,
    liquidation_proceeds_omnibus_account_id: CalaAccountId,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

impl<Perms, E> CreditFacilityLiquidationsInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        collaterals: Arc<Collaterals<Perms, E>>,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            collaterals,
            liquidation_proceeds_omnibus_account_id,
            liquidation_payment_job_spawner,
        }
    }
}

const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");
impl<Perms, E> JobInitializer for CreditFacilityLiquidationsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<crate::CoreCreditAction> + From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<crate::CoreCreditObject> + From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = CreditFacilityLiquidationsJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_LIQUIDATIONS_JOB
    }

    fn init(
        &self,
        _job: &job::Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityLiquidationsJobRunner::<Perms, E> {
            outbox: self.outbox.clone(),
            collaterals: self.collaterals.clone(),
            liquidation_proceeds_omnibus_account_id: self.liquidation_proceeds_omnibus_account_id,
            liquidation_payment_job_spawner: self.liquidation_payment_job_spawner.clone(),
        }))
    }
}

pub struct CreditFacilityLiquidationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    collaterals: Arc<Collaterals<Perms, E>>,
    liquidation_proceeds_omnibus_account_id: CalaAccountId,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityLiquidationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<crate::CoreCreditAction> + From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<crate::CoreCreditObject> + From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
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
                                .collaterals
                                .begin_op()
                                .await?;
                            self.process_message_in_op(&mut db, message.as_ref()).await?;
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

impl<Perms, E> CreditFacilityLiquidationsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<crate::CoreCreditAction> + From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<crate::CoreCreditObject> + From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "outbox.core_credit.collateral_liquidations.process_message_in_op", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(event @ CoreCreditEvent::PartialLiquidationInitiated { entity }) =
            message.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            let trigger = entity
                .liquidation_trigger
                .as_ref()
                .expect("liquidation_trigger must be set for PartialLiquidationInitiated");
            self.create_if_not_exist_in_op(
                db,
                entity.collateral_id,
                trigger.trigger_price,
                trigger.initially_expected_to_receive,
                trigger.initially_estimated_to_liquidate,
            )
            .await?;
        }
        Ok(())
    }

    #[instrument(
        name = "credit.liquidation.create_if_not_exist_in_op",
        skip(self, db),
        err
    )]
    pub async fn create_if_not_exist_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    ) -> Result<(), CollateralError> {
        if let LiquidationInitiated::Initiated {
            liquidation_id,
            secured_loan_id,
        } = self
            .collaterals
            .initiate_liquidation_in_op(
                db,
                collateral_id,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
                self.liquidation_proceeds_omnibus_account_id,
            )
            .await?
        {
            self.liquidation_payment_job_spawner
                .spawn_in_op(
                    db,
                    JobId::new(),
                    LiquidationPaymentJobConfig::<E> {
                        liquidation_id,
                        collateral_id,
                        credit_facility_id: secured_loan_id.into(),
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
        }

        Ok(())
    }
}
