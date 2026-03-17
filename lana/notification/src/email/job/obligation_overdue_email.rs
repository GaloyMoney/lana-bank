use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_access::user::Users;
use core_credit::{CoreCredit, CreditFacilityId, ObligationId, ObligationType};
use core_customer::Customers;
use job::*;
use lana_events::LanaEvent;
use money::UsdCents;
use tracing_macros::record_error_severity;

use crate::email::job::sender::EmailSenderJobSpawner;
use crate::email::templates::{EmailType, OverduePaymentEmailData};

pub const OBLIGATION_OVERDUE_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.obligation-overdue-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObligationOverdueEmailConfig {
    pub obligation_id: ObligationId,
    pub credit_facility_id: CreditFacilityId,
    pub outstanding_amount: UsdCents,
}

pub struct ObligationOverdueEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    credit: CoreCredit<Perms, LanaEvent>,
    customers: Customers<Perms, LanaEvent>,
    users: Users<Perms::Audit, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

impl<Perms> ObligationOverdueEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        credit: &CoreCredit<Perms, LanaEvent>,
        customers: &Customers<Perms, LanaEvent>,
        users: &Users<Perms::Audit, LanaEvent>,
        email_sender_job_spawner: EmailSenderJobSpawner,
    ) -> Self {
        Self {
            credit: credit.clone(),
            customers: customers.clone(),
            users: users.clone(),
            email_sender_job_spawner,
        }
    }
}

impl<Perms> JobInitializer for ObligationOverdueEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    type Config = ObligationOverdueEmailConfig;

    fn job_type(&self) -> JobType {
        OBLIGATION_OVERDUE_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationOverdueEmailRunner::<Perms> {
            config: job.config()?,
            credit: self.credit.clone(),
            customers: self.customers.clone(),
            users: self.users.clone(),
            email_sender_job_spawner: self.email_sender_job_spawner.clone(),
        }))
    }
}

struct ObligationOverdueEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: ObligationOverdueEmailConfig,
    credit: CoreCredit<Perms, LanaEvent>,
    customers: Customers<Perms, LanaEvent>,
    users: Users<Perms::Audit, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

#[async_trait]
impl<Perms> JobRunner for ObligationOverdueEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.obligation_overdue_email.run", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let obligation = self
            .credit
            .collections()
            .obligations()
            .find_by_id_without_audit_in_op(&mut op, self.config.obligation_id)
            .await?;

        let credit_facility = self
            .credit
            .facilities()
            .find_by_id_without_audit_in_op(&mut op, self.config.credit_facility_id)
            .await?;

        let party = self
            .customers
            .find_party_by_customer_id_without_audit_in_op(&mut op, credit_facility.customer_id)
            .await?;

        let email_data = OverduePaymentEmailData {
            public_id: credit_facility.public_id.to_string(),
            payment_type: match obligation.obligation_type {
                ObligationType::Disbursal => "Principal Repayment".to_string(),
                ObligationType::Interest => "Interest Payment".to_string(),
            },
            original_amount: obligation.initial_amount,
            outstanding_amount: self.config.outstanding_amount,
            due_date: obligation.due_at(),
            customer_email: party.email,
        };

        super::spawn_email_to_all_users_in_op(
            &mut op,
            &self.users,
            &self.email_sender_job_spawner,
            EmailType::OverduePayment(email_data),
        )
        .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
