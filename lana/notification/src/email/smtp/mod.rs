pub mod error;

use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

use crate::email::EmailConfig;
use error::SmtpError;

#[derive(Clone)]
pub struct SmtpClient {
    client: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
    from_name: String,
}

impl SmtpClient {
    pub fn init(config: EmailConfig) -> Result<Self, SmtpError> {
        let client = if config.insecure {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.relay)
                .port(config.port)
                .build()
        } else {
            let creds = Credentials::new(config.username, config.password);
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.relay)?
                .credentials(creds)
                .port(config.port)
                .build()
        };

        Ok(Self {
            client,
            from_email: config.from_email,
            from_name: config.from_name,
        })
    }

    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        body: String,
    ) -> Result<(), SmtpError> {
        let email = Message::builder()
            .from(Mailbox::new(
                Some(self.from_name.clone()),
                self.from_email.parse()?,
            ))
            .to(Mailbox::new(None, to_email.parse()?))
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)?;

        self.client.send(email).await?;
        Ok(())
    }
}
