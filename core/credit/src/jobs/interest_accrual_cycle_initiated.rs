use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use futures::StreamExt;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};

use crate::{CoreCreditAction, CoreCreditObject, event::CoreCreditEvent, jobs::interest_accruals};

#[derive(Serialize, Deserialize)]
pub struct InterestAccrualCycleInitiatedJobConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for InterestAccrualCycleInitiatedJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Initializer = InterestAccrualCycleInitiatedInit<Perms, E>;
}

pub struct InterestAccrualCycleInitiatedInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    pool: sqlx::PgPool,
    _phantom: std::marker::PhantomData<Perms>,
}

impl<Perms, E> InterestAccrualCycleInitiatedInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(outbox: &Outbox<E>, jobs: &Jobs, pool: &sqlx::PgPool) -> Self {
        Self {
            outbox: outbox.clone(),
            jobs: jobs.clone(),
            pool: pool.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

const INTEREST_ACCRUAL_CYCLE_INITIATED_JOB: JobType =
    JobType::new("outbox.interest-accrual-cycle-initiated");

impl<Perms, E> JobInitializer for InterestAccrualCycleInitiatedInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        INTEREST_ACCRUAL_CYCLE_INITIATED_JOB
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(InterestAccrualCycleInitiatedRunner::<Perms, E> {
            outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
            pool: self.pool.clone(),
            _phantom: std::marker::PhantomData,
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct InterestAccrualCycleInitiatedData {
    sequence: EventSequence,
}

pub struct InterestAccrualCycleInitiatedRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    pool: sqlx::PgPool,
    _phantom: std::marker::PhantomData<Perms>,
}

impl<Perms, E> InterestAccrualCycleInitiatedRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(
        name = "core_credit.interest_accrual_cycle_initiated_job.process_message",
        parent = None,
        skip(self, message),
        fields(
            seq = %message.sequence,
            handled = false,
            event_type = tracing::field::Empty,
            credit_facility_id = tracing::field::Empty,
            interest_accrual_cycle_id = tracing::field::Empty
        )
    )]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(
            event @ CoreCreditEvent::InterestAccrualCycleInitiated {
                credit_facility_id,
                interest_accrual_cycle_id,
                first_accrual_end_date,
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
            Span::current().record(
                "interest_accrual_cycle_id",
                tracing::field::display(interest_accrual_cycle_id),
            );

            let mut op = self.pool.begin().await?;
            self.jobs
                .create_and_spawn_at_in_op(
                    &mut op,
                    *interest_accrual_cycle_id,
                    interest_accruals::InterestAccrualJobConfig::<Perms, E>::new(
                        *credit_facility_id,
                    ),
                    *first_accrual_end_date,
                )
                .await?;
            op.commit().await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for InterestAccrualCycleInitiatedRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<InterestAccrualCycleInitiatedData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %INTEREST_ACCRUAL_CYCLE_INITIATED_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(msg) => {
                                    self.process_message(msg.as_ref()).await?;
                                    state.sequence = msg.sequence;
                                    current_job.update_execution_state(state).await?;
                                }
                        None => return Ok(JobCompletion::RescheduleNow)

                    }
                }
            }
        }
    }
}
