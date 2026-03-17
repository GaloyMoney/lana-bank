use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_access::user::Users;
use core_credit::{CreditFacilityId, CustomerId, PriceOfOneBTC};
use core_customer::Customers;
use job::*;
use lana_events::LanaEvent;
use money::{Satoshis, UsdCents};
use tracing_macros::record_error_severity;

use crate::email::job::sender::{EmailSenderConfig, EmailSenderJobSpawner};
use crate::email::templates::{EmailType, PartialLiquidationInitiatedEmailData};

pub const PARTIAL_LIQUIDATION_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.partial-liquidation-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialLiquidationEmailConfig {
    pub credit_facility_id: CreditFacilityId,
    pub customer_id: CustomerId,
    pub trigger_price: PriceOfOneBTC,
    pub initially_estimated_to_liquidate: Satoshis,
    pub initially_expected_to_receive: UsdCents,
}

pub struct PartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    customers: Customers<Perms, LanaEvent>,
    users: Users<Perms::Audit, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

impl<Perms> PartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        customers: &Customers<Perms, LanaEvent>,
        users: &Users<Perms::Audit, LanaEvent>,
        email_sender_job_spawner: EmailSenderJobSpawner,
    ) -> Self {
        Self {
            customers: customers.clone(),
            users: users.clone(),
            email_sender_job_spawner,
        }
    }
}

impl<Perms> JobInitializer for PartialLiquidationEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction> + From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<core_customer::CustomerObject> + From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    type Config = PartialLiquidationEmailConfig;

    fn job_type(&self) -> JobType {
        PARTIAL_LIQUIDATION_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PartialLiquidationEmailRunner::<Perms> {
            config: job.config()?,
            customers: self.customers.clone(),
            users: self.users.clone(),
            email_sender_job_spawner: self.email_sender_job_spawner.clone(),
        }))
    }
}

struct PartialLiquidationEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: PartialLiquidationEmailConfig,
    customers: Customers<Perms, LanaEvent>,
    users: Users<Perms::Audit, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

#[async_trait]
impl<Perms> JobRunner for PartialLiquidationEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<core_customer::CoreCustomerAction> + From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<core_customer::CustomerObject> + From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.partial_liquidation_email.run", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let party = self
            .customers
            .find_party_by_customer_id_without_audit_in_op(&mut op, self.config.customer_id)
            .await?;

        let email_data = PartialLiquidationInitiatedEmailData {
            facility_id: self.config.credit_facility_id.to_string(),
            trigger_price: self.config.trigger_price,
            initially_estimated_to_liquidate: self.config.initially_estimated_to_liquidate,
            initially_expected_to_receive: self.config.initially_expected_to_receive,
        };

        let email_type = EmailType::PartialLiquidationInitiated(email_data);

        super::spawn_email_to_all_users_in_op(
            &mut op,
            &self.users,
            &self.email_sender_job_spawner,
            email_type.clone(),
        )
        .await?;

        let email_config = EmailSenderConfig {
            recipient: party.email,
            email_type,
        };
        self.email_sender_job_spawner
            .spawn_in_op(&mut op, JobId::new(), email_config)
            .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
