use async_trait::async_trait;
use chrono::NaiveDate;
use domain_config::ExposedDomainConfigsReadOnly;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit::{CreditFacilityId, CustomerId, PriceOfOneBTC};
use core_customer::Customers;
use job::*;
use lana_events::LanaEvent;
use money::{Satoshis, UsdCents};
use tracing_macros::record_error_severity;

use crate::email::templates::{EmailTemplate, EmailType, UnderMarginCallEmailData};

pub const SEND_UNDER_MARGIN_CALL_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.send-under-margin-call-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendUnderMarginCallEmailConfig {
    pub credit_facility_id: CreditFacilityId,
    pub customer_id: CustomerId,
    pub effective_date: NaiveDate,
    pub collateral: Satoshis,
    pub outstanding_disbursed: UsdCents,
    pub outstanding_interest: UsdCents,
    pub price: PriceOfOneBTC,
}

pub struct SendUnderMarginCallEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    customers: Customers<Perms, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl<Perms> SendUnderMarginCallEmailInitializer<Perms>
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

impl<Perms> JobInitializer for SendUnderMarginCallEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_customer::CustomerObject>,
{
    type Config = SendUnderMarginCallEmailConfig;

    fn job_type(&self) -> JobType {
        SEND_UNDER_MARGIN_CALL_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SendUnderMarginCallEmailRunner::<Perms> {
            config: job.config()?,
            customers: self.customers.clone(),
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }
}

struct SendUnderMarginCallEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: SendUnderMarginCallEmailConfig,
    customers: Customers<Perms, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

#[async_trait]
impl<Perms> JobRunner for SendUnderMarginCallEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_customer::CustomerObject>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.send_under_margin_call_email.run", skip_all)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let party = self
            .customers
            .find_party_by_customer_id_without_audit(self.config.customer_id)
            .await?;

        let total_outstanding =
            self.config.outstanding_disbursed + self.config.outstanding_interest;
        let email_data = UnderMarginCallEmailData {
            facility_id: self.config.credit_facility_id.to_string(),
            effective: self.config.effective_date,
            collateral: self.config.collateral,
            outstanding_disbursed: self.config.outstanding_disbursed,
            outstanding_interest: self.config.outstanding_interest,
            total_outstanding,
            price: self.config.price,
        };

        super::send_rendered_email(
            &self.smtp_client,
            &self.template,
            &self.domain_configs,
            &party.email,
            &EmailType::UnderMarginCall(email_data),
        )
        .await?;

        Ok(JobCompletion::Complete)
    }
}
