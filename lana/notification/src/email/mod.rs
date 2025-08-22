pub mod config;
pub mod error;
pub mod job;

mod smtp;
pub mod templates;

use ::job::{JobId, Jobs};
use core_access::user::Users;
use core_credit::{CoreCredit, CreditFacilityId, ObligationId, ObligationType};
use core_customer::Customers;
use job::{EmailSenderConfig, EmailSenderInit};
use lana_events::LanaEvent;

use smtp::SmtpClient;
use templates::{EmailTemplate, EmailType, OverduePaymentEmailData};

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
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_credit::CoreCreditAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_customer::CoreCustomerAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_access::CoreAccessAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<governance::GovernanceAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_credit::CoreCreditObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_customer::CustomerObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_access::CoreAccessObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<governance::GovernanceObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_custody::CoreCustodyObject>,
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
        let smtp_client = SmtpClient::init(config)?;
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

        let mut query = es_entity::PaginatedQueryArgs::default();
        loop {
            let first = query.first;
            let es_entity::PaginatedQueryRet {
                entities,
                has_next_page,
                end_cursor,
            } = self
                .users
                .list_users_without_audit(query, es_entity::ListDirection::Descending)
                .await?;
            for user in entities {
                let email_config = EmailSenderConfig {
                    recipient: user.email,
                    email_type: EmailType::OverduePayment(email_data.clone()),
                };
                self.jobs
                    .create_and_spawn_in_op(op, JobId::new(), email_config)
                    .await?;
            }
            if has_next_page {
                query = es_entity::PaginatedQueryArgs {
                    first,
                    after: end_cursor,
                };
            } else {
                break;
            }
        }
        Ok(())
    }
}
