use chrono::{DateTime, Utc};
use core_money::UsdCents;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::email::error::EmailError;

#[derive(Clone, Serialize, Deserialize)]
pub enum EmailType {
    OverduePayment(OverduePaymentEmailData),
    General { subject: String, body: String },
}

#[derive(Clone)]
pub struct EmailTemplate {
    handlebars: Handlebars<'static>,
}

impl EmailTemplate {
    pub fn new() -> Result<Self, EmailError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("base", include_str!("layouts/base.hbs"))?;
        handlebars.register_template_string("styles", include_str!("partials/styles.hbs"))?;
        handlebars.register_template_string("general", include_str!("views/general.hbs"))?;
        handlebars.register_template_string("overdue", include_str!("views/overdue.hbs"))?;
        Ok(Self { handlebars })
    }

    pub fn render_email(&self, email_type: &EmailType) -> Result<(String, String), EmailError> {
        match email_type {
            EmailType::OverduePayment(data) => self.render_overdue_payment_email(data),
            EmailType::General { subject, body } => self.generic_email_template(subject, body),
        }
    }

    pub fn generic_email_template(
        &self,
        subject: &str,
        body: &str,
    ) -> Result<(String, String), EmailError> {
        let data = json!({
            "subject": subject,
            "body": body,
        });
        let html_body = self.handlebars.render("general", &data)?;
        Ok((subject.to_owned(), html_body))
    }

    fn render_overdue_payment_email(
        &self,
        data: &OverduePaymentEmailData,
    ) -> Result<(String, String), EmailError> {
        let subject = format!(
            "Lana Bank: {} Overdue Payment - {} (Facility {})",
            data.payment_type,
            data.outstanding_amount.formatted_usd(),
            data.facility_id
        );
        let data = json!({
            "subject": &subject,
            "facility_id": &data.facility_id,
            "payment_type": &data.payment_type,
            "original_amount": data.original_amount.formatted_usd(),
            "outstanding_amount": data.outstanding_amount.formatted_usd(),
            "due_date": data.due_date,
            "customer_email": &data.customer_email,
        });
        let html_body = self.handlebars.render("overdue", &data)?;
        Ok((subject, html_body))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OverduePaymentEmailData {
    pub facility_id: String,
    pub payment_type: String,
    pub original_amount: UsdCents,
    pub outstanding_amount: UsdCents,
    pub due_date: DateTime<Utc>,
    pub customer_email: String,
}
