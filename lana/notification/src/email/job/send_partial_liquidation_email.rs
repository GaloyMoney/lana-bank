use async_trait::async_trait;
use domain_config::ExposedDomainConfigsReadOnly;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_access::user::Users;
use core_credit::{CreditFacilityId, CustomerId, PriceOfOneBTC};
use core_customer::Customers;
use job::*;
use lana_events::LanaEvent;
use money::{Satoshis, UsdCents};
use tracing_macros::record_error_severity;

use crate::email::templates::{EmailTemplate, EmailType, PartialLiquidationInitiatedEmailData};

pub const SEND_PARTIAL_LIQUIDATION_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.send-partial-liquidation-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendPartialLiquidationEmailConfig {
    pub credit_facility_id: CreditFacilityId,
    pub customer_id: CustomerId,
    pub trigger_price: PriceOfOneBTC,
    pub initially_estimated_to_liquidate: Satoshis,
    pub initially_expected_to_receive: UsdCents,
}

pub struct SendPartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    customers: Customers<Perms, LanaEvent>,
    users: Users<Perms::Audit, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl<Perms> SendPartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        customers: &Customers<Perms, LanaEvent>,
        users: &Users<Perms::Audit, LanaEvent>,
        smtp_client: SmtpClient,
        template: EmailTemplate,
        domain_configs: ExposedDomainConfigsReadOnly,
    ) -> Self {
        Self {
            customers: customers.clone(),
            users: users.clone(),
            smtp_client,
            template,
            domain_configs,
        }
    }
}

impl<Perms> JobInitializer for SendPartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction> + From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<core_customer::CustomerObject> + From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    type Config = SendPartialLiquidationEmailConfig;

    fn job_type(&self) -> JobType {
        SEND_PARTIAL_LIQUIDATION_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SendPartialLiquidationEmailRunner::<Perms> {
            config: job.config()?,
            customers: self.customers.clone(),
            users: self.users.clone(),
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }
}

struct SendPartialLiquidationEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: SendPartialLiquidationEmailConfig,
    customers: Customers<Perms, LanaEvent>,
    users: Users<Perms::Audit, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

#[async_trait]
impl<Perms> JobRunner for SendPartialLiquidationEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction> + From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<core_customer::CustomerObject> + From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.send_partial_liquidation_email.run", skip_all)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let party = self
            .customers
            .find_party_by_customer_id_without_audit(self.config.customer_id)
            .await?;

        let email_data = PartialLiquidationInitiatedEmailData {
            facility_id: self.config.credit_facility_id.to_string(),
            trigger_price: self.config.trigger_price,
            initially_estimated_to_liquidate: self.config.initially_estimated_to_liquidate,
            initially_expected_to_receive: self.config.initially_expected_to_receive,
        };

        let email_type = EmailType::PartialLiquidationInitiated(email_data);

        super::send_email_to_all_users(
            &self.smtp_client,
            &self.template,
            &self.domain_configs,
            &self.users,
            &email_type,
        )
        .await?;

        if let Err(e) = super::send_rendered_email(
            &self.smtp_client,
            &self.template,
            &self.domain_configs,
            &party.email,
            &email_type,
        )
        .await
        {
            tracing::warn!(
                recipient = %party.email,
                error = %e,
                "failed to send partial liquidation email to customer"
            );
        }

        Ok(JobCompletion::Complete)
    }
}
