pub mod config;
pub mod error;
pub mod executor;
pub mod job;

mod smtp;
pub mod templates;

pub use config::EmailConfig;
use core_access::user::Users;
pub use error::EmailError;
use executor::EmailExecutor;
use job::EmailSenderJobConfig;
use job::EmailSenderJobInitializer;
use smtp::SmtpClient;
use templates::EmailTemplate;

use ::job::{JobId, Jobs};
use audit::AuditSvc;
use audit::SystemSubject;
use core_access::event::CoreAccessEvent;
use core_access::{CoreAccessAction, CoreAccessObject, UserId};
use outbox::OutboxEventMarker;
use uuid::Uuid;

pub struct EmailNotification<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    executor: EmailExecutor,
    template: EmailTemplate,
    jobs: Jobs,
    users: Users<Audit, E>,
}

impl<Audit, E> Clone for EmailNotification<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            executor: self.executor.clone(),
            template: self.template.clone(),
            jobs: self.jobs.clone(),
            users: self.users.clone(),
        }
    }
}

impl<Audit, E> EmailNotification<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub async fn init(
        jobs: &Jobs,
        config: EmailConfig,
        users: &Users<Audit, E>,
    ) -> Result<Self, EmailError> {
        let smtp_client = SmtpClient::init(config.smtp)?;
        let executor = EmailExecutor::new(smtp_client);
        let template = EmailTemplate::new()?;

        jobs.add_initializer(EmailSenderJobInitializer::new(
            executor.clone(),
            template.clone(),
        ));

        Ok(Self {
            executor,
            template,
            jobs: jobs.clone(),
            users: users.clone(),
        })
    }

    pub async fn send_obligation_overdue_notification(
        &self,
        db: &mut es_entity::DbOp<'_>,
        obligation_id: Uuid,
        credit_facility_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let subject = <Audit as AuditSvc>::Subject::system();
        let users = self.users.list_users(&subject).await?;
        for user in users {
            let email_config = EmailSenderJobConfig {
                recipient: user.email,
                subject: "Obligation overdue".to_string(),
                template_data: "latest obligation".to_string(),
            };
            self.jobs
                .create_and_spawn_in_op(db, JobId::new(), email_config)
                .await?;
        }
        Ok(())
    }
}
