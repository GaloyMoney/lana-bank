use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_deposit::{DepositAccountHolderId, DepositAccountId};
use tracing_macros::record_error_severity;

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccountCreatedNotificationConfig {
    pub account_id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
}

pub const DEPOSIT_ACCOUNT_CREATED_NOTIFICATION_COMMAND: JobType =
    JobType::new("command.notification.deposit-account-created-notification");

pub struct DepositAccountCreatedNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck,
{
    email_notification: EmailNotification<Perms>,
}

impl<Perms> DepositAccountCreatedNotificationJobInitializer<Perms>
where
    Perms: authz::PermissionCheck,
{
    pub fn new(email_notification: EmailNotification<Perms>) -> Self {
        Self { email_notification }
    }
}

impl<Perms> JobInitializer for DepositAccountCreatedNotificationJobInitializer<Perms>
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
    type Config = DepositAccountCreatedNotificationConfig;

    fn job_type(&self) -> JobType {
        DEPOSIT_ACCOUNT_CREATED_NOTIFICATION_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DepositAccountCreatedNotificationJobRunner {
            config: job.config()?,
            email_notification: self.email_notification.clone(),
        }))
    }
}

pub struct DepositAccountCreatedNotificationJobRunner<Perms>
where
    Perms: authz::PermissionCheck,
{
    config: DepositAccountCreatedNotificationConfig,
    email_notification: EmailNotification<Perms>,
}

#[async_trait]
impl<Perms> JobRunner for DepositAccountCreatedNotificationJobRunner<Perms>
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
        name = "notification.deposit_account_created_notification_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        self.email_notification
            .send_deposit_account_created_notification_in_op(
                &mut op,
                &self.config.account_id,
                &self.config.account_holder_id,
            )
            .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
