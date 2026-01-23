use chrono::{DateTime, NaiveDate, Utc};
use core_money::{Satoshis, UsdCents};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::email::error::EmailError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EmailType {
    OverduePayment(OverduePaymentEmailData),
    PartialLiquidationInitiated(PartialLiquidationInitiatedEmailData),
    UnderMarginCall(UnderMarginCallEmailData),
    DepositAccountCreated(DepositAccountCreatedEmailData),
    RoleCreated(RoleCreatedEmailData),
    General { subject: String, body: String },
}

#[derive(Clone)]
pub struct EmailTemplate {
    handlebars: Handlebars<'static>,
    admin_panel_url: Url,
}

impl EmailTemplate {
    #[allow(clippy::result_large_err)]
    pub fn new(admin_panel_url: Url) -> Result<Self, EmailError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("base", include_str!("layouts/base.hbs"))?;
        handlebars.register_template_string("styles", include_str!("partials/styles.hbs"))?;
        handlebars.register_template_string("general", include_str!("views/general.hbs"))?;
        handlebars.register_template_string("overdue", include_str!("views/overdue.hbs"))?;
        handlebars.register_template_string(
            "partial_liquidation",
            include_str!("views/partial_liquidation.hbs"),
        )?;
        handlebars.register_template_string(
            "under_margin_call",
            include_str!("views/under_margin_call.hbs"),
        )?;
        handlebars.register_template_string(
            "account_created",
            include_str!("views/account_created.hbs"),
        )?;
        handlebars
            .register_template_string("role_created", include_str!("views/role_created.hbs"))?;
        Ok(Self {
            handlebars,
            admin_panel_url,
        })
    }

    #[allow(clippy::result_large_err)]
    pub fn render_email(&self, email_type: &EmailType) -> Result<(String, String), EmailError> {
        match email_type {
            EmailType::OverduePayment(data) => self.render_overdue_payment_email(data),
            EmailType::PartialLiquidationInitiated(data) => {
                self.render_partial_liquidation_initiated_email(data)
            }
            EmailType::UnderMarginCall(data) => self.render_under_margin_call_email(data),
            EmailType::DepositAccountCreated(data) => {
                self.render_deposit_account_created_email(data)
            }
            EmailType::RoleCreated(data) => self.render_role_created_email(data),
            EmailType::General { subject, body } => self.generic_email_template(subject, body),
        }
    }

    #[allow(clippy::result_large_err)]
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

    #[allow(clippy::result_large_err)]
    fn render_overdue_payment_email(
        &self,
        data: &OverduePaymentEmailData,
    ) -> Result<(String, String), EmailError> {
        let subject = format!(
            "Lana Bank: {} Overdue Payment - {}",
            data.payment_type,
            data.outstanding_amount.formatted_usd()
        );
        let facility_url = format!(
            "{}/credit-facilities/{}",
            self.admin_panel_url, data.public_id
        );
        let data = json!({
            "subject": &subject,
            "payment_type": &data.payment_type,
            "original_amount": data.original_amount.formatted_usd(),
            "outstanding_amount": data.outstanding_amount.formatted_usd(),
            "due_date": data.due_date,
            "customer_email": &data.customer_email,
            "facility_url": &facility_url,
        });
        let html_body = self.handlebars.render("overdue", &data)?;
        Ok((subject, html_body))
    }

    #[allow(clippy::result_large_err)]
    fn render_partial_liquidation_initiated_email(
        &self,
        data: &PartialLiquidationInitiatedEmailData,
    ) -> Result<(String, String), EmailError> {
        let subject = format!(
            "Lana Bank: Partial Liquidation Initiated - {}",
            data.facility_id,
        );
        let data = json!({
            "subject": &subject,
            "facility_id": &data.facility_id,
            "trigger_price": data.trigger_price.into_inner().formatted_usd(),
            "expected_to_receive": data.initially_expected_to_receive.formatted_usd(),
            "estimated_to_liquidate": data.initially_estimated_to_liquidate.formatted_btc(),
        });
        let html_body = self.handlebars.render("partial_liquidation", &data)?;
        Ok((subject, html_body))
    }

    #[allow(clippy::result_large_err)]
    fn render_under_margin_call_email(
        &self,
        data: &UnderMarginCallEmailData,
    ) -> Result<(String, String), EmailError> {
        let subject = format!("Lana Bank: Margin Call Warning - {}", data.facility_id,);
        let data = json!({
            "subject": &subject,
            "facility_id": &data.facility_id,
            "effective": data.effective,
            "collateral": data.collateral.formatted_btc(),
            "outstanding_disbursed": data.outstanding_disbursed.formatted_usd(),
            "outstanding_interest": data.outstanding_interest.formatted_usd(),
            "total_outstanding": data.total_outstanding.formatted_usd(),
            "price": data.price.into_inner().formatted_usd(),
        });
        let html_body = self.handlebars.render("under_margin_call", &data)?;
        Ok((subject, html_body))
    }

    #[allow(clippy::result_large_err)]
    fn render_deposit_account_created_email(
        &self,
        data: &DepositAccountCreatedEmailData,
    ) -> Result<(String, String), EmailError> {
        let subject = "Lana Bank: Your Deposit Account Has Been Created".to_string();
        let data = json!({
            "subject": &subject,
            "customer_email": &data.customer_email,
            "account_id": &data.account_id,
        });
        let html_body = self.handlebars.render("account_created", &data)?;
        Ok((subject, html_body))
    }

    #[allow(clippy::result_large_err)]
    fn render_role_created_email(
        &self,
        data: &RoleCreatedEmailData,
    ) -> Result<(String, String), EmailError> {
        let subject = format!("Lana Bank: New Role Created - {}", data.role_name);
        let data = json!({
            "subject": &subject,
            "role_name": &data.role_name,
            "role_id": &data.role_id,
        });
        let html_body = self.handlebars.render("role_created", &data)?;
        Ok((subject, html_body))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverduePaymentEmailData {
    pub public_id: String,
    pub payment_type: String,
    pub original_amount: UsdCents,
    pub outstanding_amount: UsdCents,
    pub due_date: DateTime<Utc>,
    pub customer_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialLiquidationInitiatedEmailData {
    pub facility_id: String,
    pub trigger_price: core_credit::PriceOfOneBTC,
    pub initially_expected_to_receive: UsdCents,
    pub initially_estimated_to_liquidate: Satoshis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderMarginCallEmailData {
    pub facility_id: String,
    pub effective: NaiveDate,
    pub collateral: Satoshis,
    pub outstanding_disbursed: UsdCents,
    pub outstanding_interest: UsdCents,
    pub total_outstanding: UsdCents,
    pub price: core_credit::PriceOfOneBTC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAccountCreatedEmailData {
    pub account_id: String,
    pub customer_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCreatedEmailData {
    pub role_id: String,
    pub role_name: String,
}
