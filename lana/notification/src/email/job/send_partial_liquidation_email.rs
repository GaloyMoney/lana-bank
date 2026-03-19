use async_trait::async_trait;
use domain_config::ExposedDomainConfigsReadOnly;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;

use authz::PermissionCheck;
use core_credit::{CreditFacilityId, CustomerId, PriceOfOneBTC};
use job::*;
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
    pub recipient_email: String,
}

pub struct SendPartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
    _phantom: std::marker::PhantomData<Perms>,
}

impl<Perms> SendPartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        smtp_client: SmtpClient,
        template: EmailTemplate,
        domain_configs: ExposedDomainConfigsReadOnly,
    ) -> Self {
        Self {
            smtp_client,
            template,
            domain_configs,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms> JobInitializer for SendPartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
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
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
            domain_configs: self.domain_configs.clone(),
            _phantom: std::marker::PhantomData,
        }))
    }
}

struct SendPartialLiquidationEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: SendPartialLiquidationEmailConfig,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
    _phantom: std::marker::PhantomData<Perms>,
}

#[async_trait]
impl<Perms> JobRunner for SendPartialLiquidationEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.send_partial_liquidation_email.run", skip_all)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let email_data = PartialLiquidationInitiatedEmailData {
            facility_id: self.config.credit_facility_id.to_string(),
            trigger_price: self.config.trigger_price,
            initially_estimated_to_liquidate: self.config.initially_estimated_to_liquidate,
            initially_expected_to_receive: self.config.initially_expected_to_receive,
        };

        super::send_rendered_email(
            &self.smtp_client,
            &self.template,
            &self.domain_configs,
            &self.config.recipient_email,
            &EmailType::PartialLiquidationInitiated(email_data),
        )
        .await?;

        Ok(JobCompletion::Complete)
    }
}
