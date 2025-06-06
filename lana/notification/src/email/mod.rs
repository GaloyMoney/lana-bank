pub mod config;
pub mod error;
mod executor;
mod listener_job;
mod obligation_overdue_notification_job;
mod sender_job;
mod smtp;
mod templates;

use sqlx::PgPool;

pub use config::EmailConfig;
use core_access::user::Users;
pub use error::EmailError;
use executor::EmailExecutor;
use listener_job::EmailListenerJobConfig;
use listener_job::EmailListenerJobInitializer;
use obligation_overdue_notification_job::ObligationOverdueNotificationJobInitializer;
use sender_job::EmailSenderJobInitializer;
use smtp::SmtpClient;

use audit::AuditSvc;
use core_access::event::CoreAccessEvent;
use core_access::{CoreAccessAction, CoreAccessObject, UserId};
use job::Jobs;
use lana_events::LanaEvent;
use outbox::{Outbox, OutboxEventMarker};

#[derive(Clone)]
pub struct EmailNotification {
    pool: PgPool,
    executor: EmailExecutor,
}

impl EmailNotification {
    pub async fn init<Audit, E>(
        pool: &PgPool,
        jobs: &Jobs,
        outbox: &Outbox<LanaEvent>,
        config: EmailConfig,
        users: &Users<Audit, E>,
    ) -> Result<Self, EmailError>
    where
        Audit: AuditSvc,
        <Audit as AuditSvc>::Subject: From<UserId>,
        <Audit as AuditSvc>::Action: From<CoreAccessAction>,
        <Audit as AuditSvc>::Object: From<CoreAccessObject>,
        E: OutboxEventMarker<CoreAccessEvent>,
    {
        let smtp_client = SmtpClient::init(config.smtp)?;
        let notification = Self {
            pool: pool.clone(),
            executor: EmailExecutor::new(smtp_client),
        };
        jobs.add_initializer(EmailSenderJobInitializer::new());
        jobs.add_initializer(ObligationOverdueNotificationJobInitializer::new(
            users.clone(),
            jobs.clone(),
        ));
        jobs.add_initializer_and_spawn_unique(
            EmailListenerJobInitializer::<Audit, E>::new(outbox, jobs),
            EmailListenerJobConfig::<Audit, E>::new(),
        )
        .await?;

        Ok(notification)
    }

    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        template_name: &str,
        template_data: serde_json::Value,
    ) -> Result<(), EmailError> {
        println!(
            "Sending email to {}: subject={}, template={}, data={:?}",
            recipient, subject, template_name, template_data
        );
        Ok(())
    }
}
