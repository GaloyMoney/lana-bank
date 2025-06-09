use crate::email::{error::EmailError, smtp::SmtpClient, templates::EmailTemplate};

#[derive(Clone)]
pub struct EmailExecutor {
    smtp_client: SmtpClient,
}

impl EmailExecutor {
    pub fn new(smtp_client: SmtpClient) -> Self {
        Self { smtp_client }
    }

    pub async fn execute_email(
        &self,
        recipient: &str,
        subject: &str,
        body: &str,
        template: &EmailTemplate,
    ) -> Result<(), EmailError> {
        let rendered_body = template.generic_email_template(subject, body)?;
        self.smtp_client
            .send_email(recipient, subject, rendered_body)
            .await?;
        Ok(())
    }
}
