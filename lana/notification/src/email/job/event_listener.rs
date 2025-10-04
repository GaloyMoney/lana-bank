use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use lana_events::{CoreCreditEvent, LanaEvent};
use outbox::Outbox;

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize)]
pub struct PermanentEmailEventListenerConfig<AuthzType>(std::marker::PhantomData<AuthzType>);

impl<AuthzType> Default for PermanentEmailEventListenerConfig<AuthzType> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<AuthzType> JobConfig for PermanentEmailEventListenerConfig<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    type Initializer = PermanentEmailEventListenerInit<AuthzType>;
}

pub struct PermanentEmailEventListenerInit<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<AuthzType>,
}

impl<AuthzType> PermanentEmailEventListenerInit<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    pub fn new(
        outbox: &Outbox<LanaEvent>,
        email_notification: &EmailNotification<AuthzType>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            email_notification: email_notification.clone(),
        }
    }
}

const EMAIL_LISTENER_JOB: JobType = JobType::new("permanent-email-listener");
impl<AuthzType> JobInitializer for PermanentEmailEventListenerInit<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    fn job_type() -> JobType {
        EMAIL_LISTENER_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PermanentEmailEventListenerRunner {
            outbox: self.outbox.clone(),
            email_notification: self.email_notification.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct PermanentEmailEventListenerJobData {
    sequence: outbox::EventSequence,
}

pub struct PermanentEmailEventListenerRunner<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<AuthzType>,
}

#[async_trait]
impl<AuthzType> JobRunner for PermanentEmailEventListenerRunner<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PermanentEmailEventListenerJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;
        while let Some(message) = stream.next().await {
            let mut op = current_job.pool().begin().await?;
            if let Some(event) = &message.payload {
                self.handle_event(&mut op, event).await?;
            }
            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_tx(&mut op, &state)
                .await?;
            op.commit().await?;
        }
        Ok(JobCompletion::RescheduleNow)
    }
}

impl<AuthzType> PermanentEmailEventListenerRunner<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    async fn handle_event(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        event: &LanaEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let LanaEvent::Credit(CoreCreditEvent::ObligationOverdue {
            id,
            credit_facility_id,
            amount,
        }) = event
        {
            self.email_notification
                .send_obligation_overdue_notification(op, id, credit_facility_id, amount)
                .await?;
        }
        Ok(())
    }
}
