use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{Mailbox, Message, header::ContentType},
    transport::smtp::authentication::Credentials,
};

use crate::{SmtpConfig, SmtpError};

#[derive(Clone)]
pub struct SmtpClient {
    client: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpClient {
    pub fn init(config: SmtpConfig) -> Result<Self, SmtpError> {
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

        Ok(Self { client })
    }

    pub async fn send_email(
        &self,
        from_email: &str,
        from_name: Option<&str>,
        to_email: &str,
        subject: &str,
        body: String,
    ) -> Result<(), SmtpError> {
        let email = Message::builder()
            .from(Mailbox::new(
                from_name.map(str::to_string),
                from_email.parse()?,
            ))
            .to(Mailbox::new(None, to_email.parse()?))
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)?;

        self.client.send(email).await?;
        Ok(())
    }
}
