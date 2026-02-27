use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_access::RoleId;
use tracing_macros::record_error_severity;

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleCreatedNotificationConfig {
    pub role_id: RoleId,
    pub role_name: String,
}

pub const ROLE_CREATED_NOTIFICATION_COMMAND: JobType =
    JobType::new("command.notification.role-created-notification");

pub struct RoleCreatedNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck,
{
    email_notification: EmailNotification<Perms>,
}

impl<Perms> RoleCreatedNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck,
{
    pub fn new(email_notification: EmailNotification<Perms>) -> Self {
        Self { email_notification }
    }
}

impl<Perms> JobInitializer for RoleCreatedNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    type Config = RoleCreatedNotificationConfig;

    fn job_type(&self) -> JobType {
        ROLE_CREATED_NOTIFICATION_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RoleCreatedNotificationJobRunner {
            config: job.config()?,
            email_notification: self.email_notification.clone(),
        }))
    }
}

pub struct RoleCreatedNotificationJobRunner<Perms>
where
    Perms: authz::PermissionCheck,
{
    config: RoleCreatedNotificationConfig,
    email_notification: EmailNotification<Perms>,
}

#[async_trait]
impl<Perms> JobRunner for RoleCreatedNotificationJobRunner<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "notification.role_created_notification_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        self.email_notification
            .send_role_created_notification_in_op(
                &mut op,
                &self.config.role_id,
                &self.config.role_name,
            )
            .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
