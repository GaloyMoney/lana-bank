pub mod config;
pub mod error;
pub mod executor;
mod listener_job;
mod obligation_overdue_notification_job;
pub mod sender_job;
mod smtp;
pub mod templates;

pub use config::EmailConfig;
use core_access::user::Users;
pub use error::EmailError;
use executor::EmailExecutor;
use listener_job::EmailListenerJobConfig;
use listener_job::EmailListenerJobInitializer;
use obligation_overdue_notification_job::ObligationOverdueNotificationJobInitializer;
use sender_job::EmailSenderJobInitializer;
use smtp::SmtpClient;
use templates::EmailTemplate;

use audit::AuditSvc;
use core_access::event::CoreAccessEvent;
use core_access::{CoreAccessAction, CoreAccessObject, UserId};
use job::Jobs;
use lana_events::LanaEvent;
use outbox::{Outbox, OutboxEventMarker};

pub struct EmailNotification<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    _phantom: std::marker::PhantomData<(Audit, E)>,
}

impl<Audit, E> Clone for EmailNotification<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
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
        outbox: &Outbox<LanaEvent>,
        config: EmailConfig,
        users: &Users<Audit, E>,
    ) -> Result<Self, EmailError> {
        let smtp_client = SmtpClient::init(config.smtp)?;
        let executor = EmailExecutor::new(smtp_client);
        let template = EmailTemplate::new(&config.templates_path)?;
        jobs.add_initializer_and_spawn_unique(
            EmailListenerJobInitializer::new(outbox, jobs),
            EmailListenerJobConfig::<Audit, E>::new(),
        )
        .await?;
        jobs.add_initializer(ObligationOverdueNotificationJobInitializer::new(
            users, jobs,
        ));
        jobs.add_initializer(EmailSenderJobInitializer::new(executor, template));
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}
