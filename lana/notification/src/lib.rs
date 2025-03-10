pub mod email;
pub mod error;

use ::job::Jobs;
use email::{EmailConfig, EmailNotification};
use lana_events::LanaEvent;
use sqlx::PgPool;

pub type Outbox = outbox::Outbox<LanaEvent>;

#[derive(Clone)]
pub struct Notification {
    email: EmailNotification,
}

impl Notification {
    pub async fn init(
        pool: &PgPool,
        jobs: &Jobs,
        outbox: &Outbox,
        email_config: EmailConfig,
    ) -> Result<Self, error::NotificationError> {
        let email = EmailNotification::init(pool, jobs, outbox, email_config).await?;
        Ok(Self { email })
    }

    pub fn email(&self) -> &EmailNotification {
        &self.email
    }
}
