use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_access::{event::CoreAccessEvent, CoreAccessAction, CoreAccessObject, UserId};
use core_credit::{CoreCreditAction, CoreCreditObject};
use core_customer::{CoreCustomerAction, CustomerObject};
use governance::{GovernanceAction, GovernanceObject};
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use lana_events::{CoreCreditEvent, CoreCustomerEvent, GovernanceEvent, LanaEvent};
use outbox::{Outbox, OutboxEventMarker};

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize)]
pub struct EmailEventListenerConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> EmailEventListenerConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for EmailEventListenerConfig<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = EmailEventListenerInitializer<Perms, E>;
}

pub struct EmailEventListenerInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<Perms, E>,
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> EmailEventListenerInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        outbox: &Outbox<LanaEvent>,
        email_notification: &EmailNotification<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            email_notification: email_notification.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

const EMAIL_LISTENER_JOB: JobType = JobType::new("email-listener");
impl<Perms, E> JobInitializer for EmailEventListenerInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType {
        EMAIL_LISTENER_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailEventListenerRunner::<Perms, E> {
            outbox: self.outbox.clone(),
            email_notification: self.email_notification.clone(),
            _phantom: self._phantom,
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct EmailEventListenerData {
    sequence: outbox::EventSequence,
}

pub struct EmailEventListenerRunner<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<Perms, E>,
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

#[async_trait]
impl<Perms, E> JobRunner for EmailEventListenerRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + Send
        + Sync,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EmailEventListenerData>()?
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

impl<Perms, E> EmailEventListenerRunner<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    async fn handle_event(
        &self,
        db: &mut es_entity::DbOp<'_>,
        event: &LanaEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let LanaEvent::Credit(CoreCreditEvent::ObligationOverdue {
            id,
            credit_facility_id,
            amount,
        }) = event
        {
            self.email_notification
                .send_obligation_overdue_notification(db, id, credit_facility_id, amount)
                .await?;
        }
        Ok(())
    }
}
