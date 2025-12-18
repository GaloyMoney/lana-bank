use std::ops::ControlFlow;

use async_trait::async_trait;
use audit::AuditSvc;
use authz::PermissionCheck;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use futures::StreamExt as _;

use job::*;
use outbox::*;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId, LiquidationId,
    liquidation::Liquidations,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct PartialLiquidationJobData {
    sequence: EventSequence,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PartialLiquidationJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub liquidation_id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for PartialLiquidationJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = PartialLiquidationInit<Perms, E>;
}

pub struct PartialLiquidationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    liquidations: Liquidations<Perms, E>,
}

impl<Perms, E> PartialLiquidationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, liquidations: &Liquidations<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            liquidations: liquidations.clone(),
        }
    }
}

const PARTIAL_LIQUIDATION_JOB: JobType = JobType::new("outbox.partial-liquidation");
impl<Perms, E> JobInitializer for PartialLiquidationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
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
        }))
    }
}

pub struct PartialLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: PartialLiquidationJobConfig<Perms, E>,
    outbox: Outbox<E>,
    liquidations: Liquidations<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for PartialLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PartialLiquidationJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

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

                            let next = self.process_message(&mut db, message.as_ref()).await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            db.commit().await?;

                            if next.is_break() {
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
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "outbox.core_credit.partial_liquidation.process_message", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match &message.as_event() {
            Some(
                event @ PartialLiquidationRepaymentAmountReceived {
                    credit_facility_id, ..
                },
            ) if *credit_facility_id == self.config.credit_facility_id => {
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                let payment_id = crate::PaymentId::new();
                // TODO: let payment_id = credit::record_payment

                self.liquidations
                    .complete_in_op(db, self.config.liquidation_id, payment_id)
                    .await?;

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}
