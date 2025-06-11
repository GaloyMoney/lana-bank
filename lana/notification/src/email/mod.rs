pub mod config;
pub mod error;
pub mod job;

mod smtp;
pub mod templates;

use ::job::{JobId, Jobs};
use core_access::user::Users;
use core_credit::{CoreCredit, CreditFacilityId, ObligationId, ObligationType};
use core_customer::Customers;
use job::{EmailSenderConfig, EmailSenderInitializer};
use lana_events::LanaEvent;

use crate::{Authorization, LanaAudit};
use smtp::SmtpClient;
use templates::{EmailTemplate, EmailType, OverduePaymentEmailData};

pub use config::EmailConfig;
pub use error::EmailError;

#[derive(Clone)]
pub struct EmailNotification {
    jobs: Jobs,
    users: Users<LanaAudit, LanaEvent>,
    credit: CoreCredit<Authorization, LanaEvent>,
    customers: Customers<Authorization, LanaEvent>,
}

impl EmailNotification {
    pub async fn init(
        jobs: &Jobs,
        config: EmailConfig,
        users: &Users<LanaAudit, LanaEvent>,
        credit: &CoreCredit<Authorization, LanaEvent>,
        customers: &Customers<Authorization, LanaEvent>,
    ) -> Result<Self, EmailError> {
        let smtp_client = SmtpClient::init(config)?;
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
        let users = self.users.list_users_without_audit().await?;
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
