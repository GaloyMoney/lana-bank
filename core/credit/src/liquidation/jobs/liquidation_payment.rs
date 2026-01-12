use std::ops::ControlFlow;

use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use futures::StreamExt;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId, LiquidationId,
    payment::Payments,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct LiquidationPaymentJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct LiquidationPaymentJobConfig<Perms, E> {
    pub liquidation_id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for LiquidationPaymentJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            liquidation_id: self.liquidation_id,
            credit_facility_id: self.credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct LiquidationPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    payments: Payments<Perms, E>,
}

impl<Perms, E> LiquidationPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, payments: &Payments<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            payments: payments.clone(),
        }
    }
}

const LIQUIDATION_PAYMENT_JOB: JobType = JobType::new("outbox.liquidation-payment");

impl<Perms, E> JobInitializer for LiquidationPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = LiquidationPaymentJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        LIQUIDATION_PAYMENT_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(LiquidationPaymentJobRunner {
            config: job.config()?,
            outbox: self.outbox.clone(),
            payments: self.payments.clone(),
        }))
    }
}

pub type LiquidationPaymentJobSpawner<Perms, E> =
    JobSpawner<LiquidationPaymentJobConfig<Perms, E>>;

pub struct LiquidationPaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: LiquidationPaymentJobConfig<Perms, E>,
    outbox: Outbox<E>,
    payments: Payments<Perms, E>,
}

impl<Perms, E> LiquidationPaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "outbox.core_credit.liquidation_payment.process_message", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
        db: &mut es_entity::DbOp<'_>,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(
                event @ PartialLiquidationProceedsReceived {
                    amount,
                    credit_facility_id,
                    liquidation_id,
                    payment_id,
                    facility_payment_holding_account_id,
                    facility_proceeds_from_liquidation_account_id,
                    facility_uncovered_outstanding_account_id,
                    ..
                },
            ) if *liquidation_id == self.config.liquidation_id => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("payment_id", tracing::field::display(payment_id));

                let initiated_by = LedgerTransactionInitiator::System;
                let effective = chrono::Utc::now().date_naive();

                self.payments
                    .record_in_op(
                        db,
                        *payment_id,
                        *credit_facility_id,
                        crate::PaymentLedgerAccountIds {
                            facility_payment_holding_account_id:
                                *facility_payment_holding_account_id,
                            facility_uncovered_outstanding_account_id:
                                *facility_uncovered_outstanding_account_id,
                            payment_source_account_id:
                                facility_proceeds_from_liquidation_account_id.into(),
                        },
                        *amount,
                        effective,
                        initiated_by,
                    )
                    .await?;

                Ok(ControlFlow::Break(()))
            }
            Some(event @ PartialLiquidationCompleted { liquidation_id, .. })
                if *liquidation_id == self.config.liquidation_id =>
            {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                tracing::info!(
                    liquidation_id = %liquidation_id,
                    "Liquidation completed, terminating liquidation payment job"
                );

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for LiquidationPaymentJobRunner<Perms, E>
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
            .execution_state::<LiquidationPaymentJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %LIQUIDATION_PAYMENT_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self.payments.begin_op().await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message(&message, &mut db).await?;

                            db.commit().await?;

                            if next.is_break() {
                                return Ok(JobCompletion::Complete);
                            }
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
