use async_trait::async_trait;
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

use crate::{CoreCreditAction, CoreCreditEvent, CoreCreditObject};

use super::AllocateCreditFacilityPayment;

#[derive(Serialize, Deserialize, Default)]
pub struct AllocateCreditFacilityPaymentJobConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for AllocateCreditFacilityPaymentJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> AllocateCreditFacilityPaymentJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct AllocateCreditFacilityPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    process: AllocateCreditFacilityPayment<Perms, E>,
}

impl<Perms, E> AllocateCreditFacilityPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, process: &AllocateCreditFacilityPayment<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            process: process.clone(),
        }
    }
}

const ALLOCATE_CREDIT_FACILITY_PAYMENT: JobType =
    JobType::new("outbox.allocate-credit-facility-payment");

impl<Perms, E> JobInitializer for AllocateCreditFacilityPaymentInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = AllocateCreditFacilityPaymentJobConfig<Perms, E>;
    fn job_type(&self) -> JobType {
        ALLOCATE_CREDIT_FACILITY_PAYMENT
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(AllocateCreditFacilityPaymentJobRunner {
            outbox: self.outbox.clone(),
            process: self.process.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Deserialize, Serialize)]
struct AllocateCreditFacilityPaymentJobData {
    sequence: EventSequence,
}

pub struct AllocateCreditFacilityPaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    process: AllocateCreditFacilityPayment<Perms, E>,
}

impl<Perms, E> AllocateCreditFacilityPaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "core_credit.allocate_credit_facility_payment_job.process_message", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, credit_facility_id = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        if let Some(
            event @ FacilityPaymentReceived {
                credit_facility_id,
                payment_id,
                ..
            },
        ) = message.as_event()
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());
            Span::current().record(
                "credit_facility_id",
                tracing::field::display(credit_facility_id),
            );

            let initiated_by = LedgerTransactionInitiator::System;
            self.process.execute(db, *payment_id, initiated_by).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<Perms, E> JobRunner for AllocateCreditFacilityPaymentJobRunner<Perms, E>
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
            .execution_state::<AllocateCreditFacilityPaymentJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %ALLOCATE_CREDIT_FACILITY_PAYMENT,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self.process.begin_op().await?;
                            self.process_message(&mut db, &message).await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;
                            db.commit().await?;
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
