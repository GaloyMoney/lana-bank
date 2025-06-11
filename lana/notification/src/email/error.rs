use super::smtp::error::SmtpError;
use handlebars::{RenderError, TemplateError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("EmailError - SmtpError: {0}")]
    Smtp(#[from] SmtpError),
    #[error("EmailError - Template: {0}")]
    Template(#[from] TemplateError),
    #[error("EmailError - Render: {0}")]
    Render(#[from] RenderError),
    #[error("EmailError - Job: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("EmailError – User: {0}")]
    User(#[from] core_access::user::error::UserError),
    #[error("EmailError – CoreCredit: {0}")]
    CoreCredit(#[from] core_credit::error::CoreCreditError),
    #[error("EmailError – Customer: {0}")]
    Customer(#[from] core_customer::error::CustomerError),
    #[error("EmailError - CreditFacilityNotFound")]
    CreditFacilityNotFound,
    #[error("EmailError - CustomerNotFound")]
    CustomerNotFound,
}
