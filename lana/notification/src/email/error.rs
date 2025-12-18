use handlebars::{RenderError, TemplateError};
use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("EmailError - SmtpError: {0}")]
    Smtp(#[from] smtp_client::SmtpError),
    #[error("EmailError - Template: {0}")]
    Template(#[from] TemplateError),
    #[error("EmailError - Render: {0}")]
    Render(#[from] RenderError),
    #[error("EmailError - Job: {0}")]
    Job(#[from] ::job::error::JobError),
    #[error("EmailError - DomainConfig: {0}")]
    DomainConfig(#[from] domain_config::DomainConfigError),
    #[error("EmailError - User: {0}")]
    User(#[from] core_access::user::error::UserError),
    #[error("EmailError - CoreCredit: {0}")]
    CoreCredit(#[from] core_credit::error::CoreCreditError),
    #[error("EmailError - Customer: {0}")]
    Customer(#[from] core_customer::error::CustomerError),
    #[error("EmailError - Obligation: {0}")]
    Obligation(#[from] core_credit::ObligationError),
    #[error("EmailError - CreditFacility: {0}")]
    CreditFacility(#[from] core_credit::CreditFacilityError),
}

impl ErrorSeverity for EmailError {
    fn severity(&self) -> Level {
        match self {
            Self::Smtp(e) => e.severity(),
            Self::Template(_) => Level::ERROR,
            Self::Render(_) => Level::ERROR,
            Self::Job(_) => Level::ERROR,
            Self::DomainConfig(e) => e.severity(),
            Self::User(e) => e.severity(),
            Self::CoreCredit(e) => e.severity(),
            Self::Customer(e) => e.severity(),
            Self::Obligation(e) => e.severity(),
            Self::CreditFacility(e) => e.severity(),
        }
    }
}
