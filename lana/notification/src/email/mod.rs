pub mod config;
pub mod error;
pub mod event;
mod executor;
mod job;
mod smtp;
mod templates;

use ::job::Jobs;
use outbox::Outbox;
use sqlx::{PgPool, Postgres, Transaction};

pub use config::EmailConfig;
pub use error::EmailError;
pub use event::EmailEvent;
use executor::EmailExecutor;
use job::{EmailJobConfig, EmailJobInitializer};
use smtp::SmtpClient;

pub type EmailOutbox = Outbox<EmailEvent>;

#[derive(Clone)]
pub struct EmailNotification {
    pool: PgPool,
    outbox: EmailOutbox,
}

impl EmailNotification {
    pub async fn init(
        pool: &PgPool,
        jobs: &Jobs,
        outbox: &EmailOutbox,
        config: EmailConfig,
    ) -> Result<Self, EmailError> {
        let smtp = SmtpClient::init(config.smtp)?;
        let executor = EmailExecutor::new(smtp);

        jobs.add_initializer_and_spawn_unique(
            EmailJobInitializer::new(pool, outbox, executor),
            EmailJobConfig,
        )
        .await?;

        Ok(Self {
            pool: pool.clone(),
            outbox: outbox.clone(),
        })
    }

    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        template_name: &str,
        template_data: serde_json::Value,
    ) -> Result<(), EmailError> {
        let mut tx = self.pool.begin().await.map_err(EmailError::Database)?;

        let result = self
            .send_email_in_tx(&mut tx, recipient, subject, template_name, template_data)
            .await;

        if result.is_ok() {
            tx.commit().await.map_err(EmailError::Database)?;
        }

        result
    }

    pub async fn send_email_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        recipient: &str,
        subject: &str,
        template_name: &str,
        template_data: serde_json::Value,
    ) -> Result<(), EmailError> {
        let email_event = EmailEvent::Requested {
            id: uuid::Uuid::new_v4(),
            recipient: recipient.to_string(),
            subject: subject.to_string(),
            template_name: template_name.to_string(),
            template_data,
            timestamp: chrono::Utc::now(),
        };

        self.outbox
            .publish_persisted(tx, email_event)
            .await
            .map_err(|e| EmailError::Outbox(e.to_string()))?;

        Ok(())
    }
}
