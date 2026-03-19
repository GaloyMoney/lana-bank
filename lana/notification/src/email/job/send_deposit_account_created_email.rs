use async_trait::async_trait;
use domain_config::ExposedDomainConfigsReadOnly;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::Customers;
use core_deposit::{DepositAccountHolderId, DepositAccountId};
use job::*;
use lana_events::LanaEvent;
use tracing_macros::record_error_severity;

use crate::email::templates::{DepositAccountCreatedEmailData, EmailTemplate, EmailType};

pub const SEND_DEPOSIT_ACCOUNT_CREATED_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.send-deposit-account-created-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendDepositAccountCreatedEmailConfig {
    pub account_id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
}

pub struct SendDepositAccountCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    customers: Customers<Perms, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl<Perms> SendDepositAccountCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        customers: &Customers<Perms, LanaEvent>,
        smtp_client: SmtpClient,
        template: EmailTemplate,
        domain_configs: ExposedDomainConfigsReadOnly,
    ) -> Self {
        Self {
            customers: customers.clone(),
            smtp_client,
            template,
            domain_configs,
        }
    }
}

impl<Perms> JobInitializer for SendDepositAccountCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_customer::CustomerObject>,
{
    type Config = SendDepositAccountCreatedEmailConfig;

    fn job_type(&self) -> JobType {
        SEND_DEPOSIT_ACCOUNT_CREATED_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SendDepositAccountCreatedEmailRunner::<Perms> {
            config: job.config()?,
            customers: self.customers.clone(),
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }
}

struct SendDepositAccountCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: SendDepositAccountCreatedEmailConfig,
    customers: Customers<Perms, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

#[async_trait]
impl<Perms> JobRunner for SendDepositAccountCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_customer::CustomerObject>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.send_deposit_account_created_email.run", skip_all)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let customer_id: core_customer::CustomerId = self.config.account_holder_id.into();
        let party = self
            .customers
            .find_party_by_customer_id_without_audit(customer_id)
            .await?;

        let email_data = DepositAccountCreatedEmailData {
            account_id: self.config.account_id.to_string(),
            customer_email: party.email.clone(),
        };

        super::send_rendered_email(
            &self.smtp_client,
            &self.template,
            &self.domain_configs,
            &party.email,
            &EmailType::DepositAccountCreated(email_data),
        )
        .await?;

        Ok(JobCompletion::Complete)
    }
}
