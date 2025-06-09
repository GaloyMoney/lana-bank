use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use lana_events::{CoreCreditEvent, LanaEvent};
use outbox::Outbox;

use crate::email::EmailNotification;
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
    email_notification: EmailNotification<Audit, E>,
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
    pub fn new(
        outbox: &Outbox<LanaEvent>,
        email_notification: &EmailNotification<Audit, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            email_notification: email_notification.clone(),
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
            email_notification: self.email_notification.clone(),
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
    email_notification: EmailNotification<Audit, E>,
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
            let mut db = self.email_notification.jobs.begin_op().await?;
            if let Some(event) = &message.payload {
                self.handle_event(&mut db, event).await?;
            }
            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_tx(db.tx(), &state)
                .await?;
            db.commit().await?;
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
    async fn handle_event(
        &self,
        db: &mut es_entity::DbOp<'_>,
        event: &LanaEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            LanaEvent::Credit(CoreCreditEvent::ObligationOverdue {
                id,
                credit_facility_id,
                ..
            }) => {
                self.email_notification
                    .send_obligation_overdue_notification(db, id.into(), credit_facility_id.into())
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
