pub mod config;
pub mod error;
mod executor;
mod listener_job;
mod sender_job;
mod smtp;
mod templates;

use sqlx::PgPool;

pub use config::EmailConfig;
pub use error::EmailError;
use executor::EmailExecutor;
use listener_job::EmailListenerJobInitializer;
use listener_job::EmailListenerJobConfig;
use sender_job::EmailSenderJobInitializer;
use smtp::SmtpClient;

use outbox::Outbox;
use job::Jobs;
use lana_events::LanaEvent;

#[derive(Clone)]
pub struct EmailNotification {
    pool: PgPool,
    executor: EmailExecutor,
}

impl EmailNotification {
    pub async fn init(
        pool: &PgPool,
        jobs: &Jobs,
        outbox: &Outbox<LanaEvent>,
        config: EmailConfig,
    ) -> Result<Self, EmailError> {
        let smtp_client = SmtpClient::init(config.smtp)?;

        let notification = Self {
            pool: pool.clone(),
            executor: EmailExecutor::new(smtp_client),
        };

        jobs.add_initializer(EmailSenderJobInitializer::new(notification.clone()));
        jobs.add_initializer_and_spawn_unique(
            EmailListenerJobInitializer::new(pool, outbox, jobs),
            EmailListenerJobConfig,
        )
        .await?;

        Ok(notification)
    }

    pub async fn send_customer_welcome(&self, customer_id: &uuid::Uuid) -> Result<(), EmailError> {
        let recipient = "test@email.com";
        let template_data = serde_json::json!({
            "customer_id": customer_id.to_string(),
            "welcome_message": "Welcome to Lana!",
        });

        self.send_email(
            &recipient,
            "Welcome to Lana!",
            "customer_welcome",
            template_data,
        )
        .await
    }

    async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        template_name: &str,
        template_data: serde_json::Value,
    ) -> Result<(), EmailError> {
        // let template = EmailTemplate::new(&self.templates_path)?;
        // self.executor.execute_email(recipient, subject, template_name, &template_data, template);

        unimplemented!();
        Ok(())
    }
}
