use handlebars::Handlebars;
use serde_json::json;

use crate::email::error::EmailError;

#[derive(Clone)]
pub struct EmailTemplate {
    handlebars: Handlebars<'static>,
}

impl EmailTemplate {
    pub fn new() -> Result<Self, EmailError> {
        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string("base", include_str!("layouts/base.hbs"))
            .map_err(|e| EmailError::Template(e.to_string()))?;

        handlebars
            .register_template_string("general", include_str!("general.hbs"))
            .map_err(|e| EmailError::Template(e.to_string()))?;

        handlebars
            .register_template_string("styles", include_str!("partials/styles.hbs"))
            .map_err(|e| EmailError::Template(e.to_string()))?;

        Ok(Self { handlebars })
    }

    pub fn render(
        &self,
        template_name: &str,
        context: &serde_json::Value,
    ) -> Result<String, EmailError> {
        self.handlebars
            .render(template_name, context)
            .map_err(|e| EmailError::Template(e.to_string()))
    }

    pub fn generic_email_template(&self, subject: &str, body: &str) -> Result<String, EmailError> {
        let data = json!({
            "subject": subject,
            "body": body,
        });
        self.handlebars
            .render("general", &data)
            .map_err(|e| EmailError::Template(e.to_string()))
    }
}
