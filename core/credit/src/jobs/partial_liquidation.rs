use std::ops::ControlFlow;

use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use job::*;
use obix::EventSequence;
use obix::out::*;

use crate::CreditFacilities;
use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CoreCustodyAction, CoreCustodyEvent,
    CoreCustodyObject, CreditFacilityId, GovernanceAction, GovernanceEvent, GovernanceObject,
    LiquidationId, Obligations, Payments, liquidation::Liquidations,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct PartialLiquidationJobData {
    sequence: EventSequence,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PartialLiquidationJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub liquidation_id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for PartialLiquidationJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = PartialLiquidationInit<Perms, E>;
}

pub struct PartialLiquidationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    liquidations: Liquidations<Perms, E>,
    payments: Payments<Perms>,
    obligations: Obligations<Perms, E>,
    facilities: CreditFacilities<Perms, E>,
}

impl<Perms, E> PartialLiquidationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        liquidations: &Liquidations<Perms, E>,
        payments: &Payments<Perms>,
        obligations: &Obligations<Perms, E>,
        facilities: &CreditFacilities<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            liquidations: liquidations.clone(),
            payments: payments.clone(),
            obligations: obligations.clone(),
            facilities: facilities.clone(),
        }
    }
}

const PARTIAL_LIQUIDATION_JOB: JobType = JobType::new("outbox.partial-liquidation");
impl<Perms, E> JobInitializer for PartialLiquidationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        PARTIAL_LIQUIDATION_JOB
    }

    fn init(&self, job: &job::Job) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PartialLiquidationJobRunner::<Perms, E> {
            config: job.config()?,
            outbox: self.outbox.clone(),
            liquidations: self.liquidations.clone(),
            payments: self.payments.clone(),
            obligations: self.obligations.clone(),
            facilities: self.facilities.clone(),
        }))
    }
}

pub struct PartialLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    config: PartialLiquidationJobConfig<Perms, E>,
    outbox: Outbox<E>,
    liquidations: Liquidations<Perms, E>,
    payments: Payments<Perms>,
    obligations: Obligations<Perms, E>,
    facilities: CreditFacilities<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for PartialLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PartialLiquidationJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %PARTIAL_LIQUIDATION_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self.liquidations.begin_op().await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message(&mut db, message.as_ref()).await?;

                            db.commit().await?;

                            if next.is_break() {
                                // If the partial liquidation has been completed,
                                // terminate the job, too.
                                return Ok(JobCompletion::Complete);
                            }
                        }
                        None => return Ok(JobCompletion::RescheduleNow)
                    }
                }
            }
        }
    }
}

impl<Perms, E> PartialLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "outbox.core_credit.partial_liquidation.process_message", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match &message.as_event() {
            Some(
                event @ PartialLiquidationProceedsReceived {
                    amount,
                    credit_facility_id,
                    liquidation_id,
                    payment_id,
                    facility_payment_holding_account_id,
                    facility_proceeds_from_liquidation_account_id,
                    ..
                },
            ) if *liquidation_id == self.config.liquidation_id => {
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("payment_id", tracing::field::display(payment_id));

                let initiated_by = LedgerTransactionInitiator::System;

                let effective = crate::time::now().date_naive();

                if let Some(payment) = self
                    .payments
                    .record_in_op(
                        db,
                        *payment_id,
                        *credit_facility_id,
                        *facility_payment_holding_account_id,
                        *facility_proceeds_from_liquidation_account_id,
                        *amount,
                        effective,
                        initiated_by,
                    )
                    .await?
                {
                    self.obligations
                        .allocate_payment_in_op(db, &payment, initiated_by)
                        .await?;

                    self.facilities
                        .complete_liquidation_in_op(
                            db,
                            *credit_facility_id,
                            self.config.liquidation_id,
                        )
                        .await?;

                    self.liquidations
                        .complete_in_op(db, self.config.liquidation_id, *payment_id)
                        .await?;
                }

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}
