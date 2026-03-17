use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit::{CreditFacilityId, CustomerId, PriceOfOneBTC};
use core_customer::Customers;
use job::*;
use lana_events::LanaEvent;
use money::{Satoshis, UsdCents};
use tracing_macros::record_error_severity;

use crate::email::job::sender::{EmailSenderConfig, EmailSenderJobSpawner};
use crate::email::templates::{EmailType, UnderMarginCallEmailData};

pub const UNDER_MARGIN_CALL_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.under-margin-call-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnderMarginCallEmailConfig {
    pub credit_facility_id: CreditFacilityId,
    pub customer_id: CustomerId,
    pub effective_date: NaiveDate,
    pub collateral: Satoshis,
    pub outstanding_disbursed: UsdCents,
    pub outstanding_interest: UsdCents,
    pub price: PriceOfOneBTC,
}

pub struct UnderMarginCallEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    customers: Customers<Perms, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

impl<Perms> UnderMarginCallEmailInitializer<Perms>
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

impl<Perms> JobInitializer for UnderMarginCallEmailInitializer<Perms>
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
    type Config = UnderMarginCallEmailConfig;

    fn job_type(&self) -> JobType {
        UNDER_MARGIN_CALL_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UnderMarginCallEmailRunner::<Perms> {
            config: job.config()?,
            customers: self.customers.clone(),
            email_sender_job_spawner: self.email_sender_job_spawner.clone(),
        }))
    }
}

struct UnderMarginCallEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: UnderMarginCallEmailConfig,
    customers: Customers<Perms, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

#[async_trait]
impl<Perms> JobRunner for UnderMarginCallEmailRunner<Perms>
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
    #[tracing::instrument(name = "notification.under_margin_call_email.run", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let party = self
            .customers
            .find_party_by_customer_id_without_audit_in_op(&mut op, self.config.customer_id)
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

        let email_config = EmailSenderConfig {
            recipient: party.email,
            email_type: EmailType::UnderMarginCall(email_data),
        };
        self.email_sender_job_spawner
            .spawn_in_op(&mut op, JobId::new(), email_config)
            .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
