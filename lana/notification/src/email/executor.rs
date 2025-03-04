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
        template_name: &str,
        template_data: &serde_json::Value,
        template: &EmailTemplate,
    ) -> Result<(), EmailError> {
        let body = template.render(template_name, template_data)?;

        self.smtp_client
            .send_email(recipient, subject, body)
            .await
            .map_err(EmailError::from)
    }
}
