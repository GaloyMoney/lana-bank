pub mod email;
pub mod error;

use ::job::Jobs;
use core_access::user::Users;
use email::job::{EmailListenerJobConfig, EmailListenerJobInitializer};
use email::{EmailConfig, EmailNotification};
use lana_events::LanaEvent;

use audit::AuditSvc;
use core_access::event::CoreAccessEvent;
use core_access::{CoreAccessAction, CoreAccessObject, UserId};
use outbox::OutboxEventMarker;

pub type NotificationOutbox = outbox::Outbox<LanaEvent>;

pub struct Notification<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    email: EmailNotification<Audit, E>,
}

impl<Audit, E> Clone for Notification<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            email: self.email.clone(),
        }
    }
}

impl<Audit, E> Notification<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub async fn init(
        jobs: &Jobs,
        outbox: &NotificationOutbox,
        email_config: EmailConfig,
        users: &Users<Audit, E>,
    ) -> Result<Self, error::NotificationError> {
        let email = EmailNotification::init(jobs, email_config, users).await?;
        jobs.add_initializer_and_spawn_unique(
            EmailListenerJobInitializer::new(outbox, &email),
            EmailListenerJobConfig::new(),
        )
        .await?;

        Ok(Self { email })
    }
}
