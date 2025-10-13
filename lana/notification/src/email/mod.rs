pub mod config;
pub mod error;
pub mod job;
pub mod templates;

use ::job::{JobId, Jobs};
use core_access::user::Users;
use core_credit::{CoreCredit, CreditFacilityId, ObligationId, ObligationType};
use core_customer::Customers;
use job::{EmailSenderConfig, EmailSenderInit};
use lana_events::LanaEvent;
use smtp_client::SmtpClient;

use templates::{
    DepositAccountCreatedEmailData, EmailTemplate, EmailType, OverduePaymentEmailData,
};

pub use config::EmailConfig;
pub use error::EmailError;

#[derive(Clone)]
pub struct EmailNotification<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    jobs: Jobs,
    users: Users<AuthzType::Audit, LanaEvent>,
    credit: CoreCredit<AuthzType, LanaEvent>,
    customers: Customers<AuthzType, LanaEvent>,
    _authz: std::marker::PhantomData<AuthzType>,
}

impl<AuthzType> EmailNotification<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    pub async fn init(
        jobs: &Jobs,
        config: EmailConfig,
        users: &Users<AuthzType::Audit, LanaEvent>,
        credit: &CoreCredit<AuthzType, LanaEvent>,
        customers: &Customers<AuthzType, LanaEvent>,
    ) -> Result<Self, EmailError> {
        let template = EmailTemplate::new(config.admin_panel_url.clone())?;
        let smtp_client = SmtpClient::init(config.to_smtp_config())?;
        jobs.add_initializer(EmailSenderInit::new(smtp_client, template));
        Ok(Self {
            jobs: jobs.clone(),
            users: users.clone(),
            credit: credit.clone(),
            customers: customers.clone(),
            _authz: std::marker::PhantomData,
        })
    }

    pub async fn send_obligation_overdue_notification(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        obligation_id: &ObligationId,
        credit_facility_id: &CreditFacilityId,
        amount: &core_money::UsdCents,
    ) -> Result<(), EmailError> {
        let obligation = self
            .credit
            .obligations()
            .find_by_id_without_audit(*obligation_id)
            .await?;

        let credit_facility = self
            .credit
            .facilities()
            .find_by_id_without_audit(*credit_facility_id)
            .await?;

        let customer = self
            .customers
            .find_by_id_without_audit(credit_facility.customer_id)
            .await?;

        let email_data = OverduePaymentEmailData {
            facility_id: credit_facility_id.to_string(),
            payment_type: match obligation.obligation_type {
                ObligationType::Disbursal => "Principal Repayment".to_string(),
                ObligationType::Interest => "Interest Payment".to_string(),
            },
            original_amount: obligation.initial_amount,
            outstanding_amount: *amount,
            due_date: obligation.due_at(),
            customer_email: customer.email,
        };

        let mut has_next_page = true;
        let mut after = None;
        // currently email notifications are sent to all users in the system
        // TODO: create a role for receiving margin call / overdue payment emails
        while has_next_page {
            let es_entity::PaginatedQueryRet {
                entities: users,
                has_next_page: next_page,
                end_cursor,
            } = self
                .users
                .list_users_without_audit(
                    es_entity::PaginatedQueryArgs { first: 20, after },
                    es_entity::ListDirection::Descending,
                )
                .await?;
            (after, has_next_page) = (end_cursor, next_page);

            for user in users {
                let email_config = EmailSenderConfig {
                    recipient: user.email,
                    email_type: EmailType::OverduePayment(email_data.clone()),
                };
                self.jobs
                    .create_and_spawn_in_op(op, JobId::new(), email_config)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn send_deposit_account_created_notification(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        account_id: &core_deposit::DepositAccountId,
        account_holder_id: &core_deposit::DepositAccountHolderId,
    ) -> Result<(), EmailError> {
        let customer_id: core_customer::CustomerId = (*account_holder_id).into();
        let customer = self.customers.find_by_id_without_audit(customer_id).await?;

        let email_data = DepositAccountCreatedEmailData {
            account_id: account_id.to_string(),
            customer_email: customer.email.clone(),
        };

        let email_config = EmailSenderConfig {
            recipient: customer.email,
            email_type: EmailType::DepositAccountCreated(email_data),
        };

        self.jobs
            .create_and_spawn_in_op(op, JobId::new(), email_config)
            .await?;

        Ok(())
    }
}
