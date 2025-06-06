use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobId, JobInitializer, JobRunner, JobType, Jobs,
    RetrySettings,
};
use lana_events::{CoreCreditEvent, LanaEvent};
use outbox::Outbox;

use crate::email::obligation_overdue_notification_job::ObligationOverdueNotificationJobConfig;
use audit::AuditSvc;
use core_access::event::CoreAccessEvent;
use core_access::{CoreAccessAction, CoreAccessObject, UserId};
use outbox::OutboxEventMarker;

#[derive(Serialize, Deserialize)]
pub struct EmailListenerJobConfig<Audit, E> {
    _phantom: std::marker::PhantomData<(Audit, E)>,
}
impl<Audit, E> EmailListenerJobConfig<Audit, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Audit, E> JobConfig for EmailListenerJobConfig<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    type Initializer = EmailListenerJobInitializer<Audit, E>;
}

pub struct EmailListenerJobInitializer<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<LanaEvent>,
    jobs: Jobs,
    _phantom: std::marker::PhantomData<(Audit, E)>,
}

impl<Audit, E> EmailListenerJobInitializer<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(outbox: &Outbox<LanaEvent>, jobs: &Jobs) -> Self {
        Self {
            outbox: outbox.clone(),
            jobs: jobs.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

const EMAIL_LISTENER_JOB: JobType = JobType::new("email-listener");
impl<Audit, E> JobInitializer for EmailListenerJobInitializer<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn job_type() -> JobType {
        EMAIL_LISTENER_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailListenerJobRunner::<Audit, E> {
            outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
            _phantom: self._phantom.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct EmailListenerJobData {
    sequence: outbox::EventSequence,
}

pub struct EmailListenerJobRunner<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<LanaEvent>,
    jobs: Jobs,
    _phantom: std::marker::PhantomData<(Audit, E)>,
}

#[async_trait]
impl<Audit, E> JobRunner for EmailListenerJobRunner<Audit, E>
where
    Audit: AuditSvc + Send + Sync,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent> + Send + Sync,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EmailListenerJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;
        while let Some(message) = stream.next().await {
            if let Some(event) = &message.payload {
                self.handle_event(event).await?;
            }
            state.sequence = message.sequence;
            current_job.update_execution_state(state.clone()).await?;
        }
        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Audit, E> EmailListenerJobRunner<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    async fn handle_event(&self, event: &LanaEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            LanaEvent::Credit(CoreCreditEvent::ObligationOverdue {
                id,
                credit_facility_id,
                ..
            }) => {
                let config = ObligationOverdueNotificationJobConfig::new(
                    id.into(),
                    credit_facility_id.into(),
                );
                let mut db = self.jobs.begin_op().await?;
                self.jobs
                    .create_and_spawn_in_op::<ObligationOverdueNotificationJobConfig<Audit, E>>(
                        &mut db,
                        JobId::new(),
                        config,
                    )
                    .await?;
                db.commit().await?;
            }
            _ => {}
        }
        Ok(())
    }
}
