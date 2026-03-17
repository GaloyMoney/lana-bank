use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::Customers;
use core_deposit::{DepositAccountHolderId, DepositAccountId};
use job::*;
use lana_events::LanaEvent;
use tracing_macros::record_error_severity;

use crate::email::job::sender::{EmailSenderConfig, EmailSenderJobSpawner};
use crate::email::templates::{DepositAccountCreatedEmailData, EmailType};

pub const DEPOSIT_ACCOUNT_CREATED_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.deposit-account-created-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccountCreatedEmailConfig {
    pub account_id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
}

pub struct DepositAccountCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    customers: Customers<Perms, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

impl<Perms> DepositAccountCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        customers: &Customers<Perms, LanaEvent>,
        email_sender_job_spawner: EmailSenderJobSpawner,
    ) -> Self {
        Self {
            customers: customers.clone(),
            email_sender_job_spawner,
        }
    }
}

impl<Perms> JobInitializer for DepositAccountCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    type Config = DepositAccountCreatedEmailConfig;

    fn job_type(&self) -> JobType {
        DEPOSIT_ACCOUNT_CREATED_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DepositAccountCreatedEmailRunner::<Perms> {
            config: job.config()?,
            customers: self.customers.clone(),
            email_sender_job_spawner: self.email_sender_job_spawner.clone(),
        }))
    }
}

struct DepositAccountCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: DepositAccountCreatedEmailConfig,
    customers: Customers<Perms, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

#[async_trait]
impl<Perms> JobRunner for DepositAccountCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.deposit_account_created_email.run", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let customer_id: core_customer::CustomerId = self.config.account_holder_id.into();
        let party = self
            .customers
            .find_party_by_customer_id_without_audit_in_op(&mut op, customer_id)
            .await?;

        let email_data = DepositAccountCreatedEmailData {
            account_id: self.config.account_id.to_string(),
            customer_email: party.email.clone(),
        };

        let email_config = EmailSenderConfig {
            recipient: party.email,
            email_type: EmailType::DepositAccountCreated(email_data),
        };
        self.email_sender_job_spawner
            .spawn_in_op(&mut op, JobId::new(), email_config)
            .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
