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
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId, Obligations, Payments,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityPaymentAllocationJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityPaymentAllocationJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for CreditFacilityPaymentAllocationJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            credit_facility_id: self.credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct CreditFacilityPaymentAllocationInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    payments: Payments<Perms, E>,
    obligations: Obligations<Perms, E>,
}

impl<Perms, E> CreditFacilityPaymentAllocationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        payments: &Payments<Perms, E>,
        obligations: &Obligations<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            payments: payments.clone(),
            obligations: obligations.clone(),
        }
    }
}

const CREDIT_FACILITY_PAYMENT_ALLOCATION_JOB: JobType =
    JobType::new("outbox.credit-facility-payment-allocation");

impl<Perms, E> JobInitializer for CreditFacilityPaymentAllocationInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = CreditFacilityPaymentAllocationJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_PAYMENT_ALLOCATION_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityPaymentAllocationJobRunner {
            config: job.config()?,
            outbox: self.outbox.clone(),
            payments: self.payments.clone(),
            obligations: self.obligations.clone(),
        }))
    }
}

pub type CreditFacilityPaymentAllocationJobSpawner<Perms, E> =
    JobSpawner<CreditFacilityPaymentAllocationJobConfig<Perms, E>>;

pub struct CreditFacilityPaymentAllocationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityPaymentAllocationJobConfig<Perms, E>,
    outbox: Outbox<E>,
    payments: Payments<Perms, E>,
    obligations: Obligations<Perms, E>,
}

impl<Perms, E> CreditFacilityPaymentAllocationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "outbox.core_credit.payment_allocation_job.process_message", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, credit_facility_id = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(
                event @ FacilityPaymentReceived {
                    credit_facility_id,
                    payment_id,
                    ..
                },
            ) if *credit_facility_id == self.config.credit_facility_id => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record(
                    "credit_facility_id",
                    tracing::field::display(credit_facility_id),
                );

                let initiated_by = LedgerTransactionInitiator::System;

                if let Some(payment) = self.payments.find_by_id(*payment_id).await? {
                    self.obligations
                        .allocate_payment_in_op(db, &payment, initiated_by)
                        .await?;
                }

                Ok(ControlFlow::Continue(()))
            }
            Some(
                event @ FacilityCompleted {
                    id: credit_facility_id,
                    ..
                },
            ) if *credit_facility_id == self.config.credit_facility_id => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record(
                    "credit_facility_id",
                    tracing::field::display(credit_facility_id),
                );

                tracing::info!(
                    credit_facility_id = %credit_facility_id,
                    "Facility completed, terminating payment allocation job"
                );

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for CreditFacilityPaymentAllocationJobRunner<Perms, E>
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
            .execution_state::<CreditFacilityPaymentAllocationJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %CREDIT_FACILITY_PAYMENT_ALLOCATION_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self.obligations.begin_op().await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message(&mut db, &message).await?;

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
