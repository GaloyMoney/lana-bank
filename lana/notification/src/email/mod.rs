pub mod config;
pub mod error;
pub mod job;

mod smtp;
pub mod templates;

pub use config::EmailConfig;
pub use error::EmailError;

use ::job::{JobId, Jobs};
use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_access::{
    event::CoreAccessEvent, user::Users, CoreAccessAction, CoreAccessObject, UserId,
};
use core_credit::{
    error::CoreCreditError, CoreCredit, CoreCreditAction, CoreCreditObject, CreditFacilityId,
    ObligationId, ObligationType,
};
use core_customer::{CoreCustomerAction, CustomerObject, Customers};
use governance::{GovernanceAction, GovernanceObject};
use job::{EmailSenderConfig, EmailSenderInitializer};
use lana_events::{CoreCreditEvent, CoreCustomerEvent, GovernanceEvent};
use outbox::OutboxEventMarker;
use smtp::SmtpClient;
use templates::{EmailTemplate, EmailType, OverduePaymentEmailData};

pub struct EmailNotification<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    jobs: Jobs,
    users: Users<Perms::Audit, E>,
    credit: CoreCredit<Perms, E>,
    customers: Customers<Perms, E>,
}

impl<Perms, E> Clone for EmailNotification<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            jobs: self.jobs.clone(),
            users: self.users.clone(),
            credit: self.credit.clone(),
            customers: self.customers.clone(),
        }
    }
}

impl<Perms, E> EmailNotification<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn init(
        jobs: &Jobs,
        config: EmailConfig,
        users: &Users<Perms::Audit, E>,
        credit: &CoreCredit<Perms, E>,
        customers: &Customers<Perms, E>,
    ) -> Result<Self, EmailError> {
        let smtp_client = SmtpClient::init(config.smtp)?;
        let template = EmailTemplate::new()?;

        jobs.add_initializer(EmailSenderInitializer::new(smtp_client, template));

        Ok(Self {
            jobs: jobs.clone(),
            users: users.clone(),
            credit: credit.clone(),
            customers: customers.clone(),
        })
    }

    pub async fn send_obligation_overdue_notification(
        &self,
        db: &mut es_entity::DbOp<'_>,
        obligation_id: &ObligationId,
        credit_facility_id: &CreditFacilityId,
        amount: &core_money::UsdCents,
    ) -> Result<(), EmailError> {
        let subject = <Perms::Audit as AuditSvc>::Subject::system();
        let users = self.users.list_users(&subject).await?;

        let obligation = self
            .credit
            .obligations()
            .find_by_id(*obligation_id)
            .await
            .map_err(CoreCreditError::from)?;

        let credit_facility = self
            .credit
            .facilities()
            .find_by_id(&subject, *credit_facility_id)
            .await
            .map_err(CoreCreditError::from)?
            .ok_or(EmailError::CreditFacilityNotFound)?;

        let customer = self
            .customers
            .find_by_id(&subject, credit_facility.customer_id)
            .await?
            .ok_or(EmailError::CustomerNotFound)?;

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

        for user in users {
            let email_config = EmailSenderConfig {
                recipient: user.email,
                email_type: EmailType::OverduePayment(email_data.clone()),
            };
            self.jobs
                .create_and_spawn_in_op(db, JobId::new(), email_config)
                .await?;
        }
        Ok(())
    }
}
