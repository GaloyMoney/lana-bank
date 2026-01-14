use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use lana_app::{
    access::user::UserError,
    app::ApplicationError,
    credit::{error::CoreCreditError, terms_template_error::TermsTemplateError},
    customer::error::CustomerError,
    deposit::error::CoreDepositError,
    primitives::ConversionError,
};

#[derive(Error, Debug)]
pub enum SimBootstrapError {
    #[error("SimBootstrapError - User: {0}")]
    User(#[from] UserError),
    #[error("SimBootstrapError - Customer: {0}")]
    Customer(#[from] CustomerError),
    #[error("SimBootstrapError - Deposit: {0}")]
    Deposit(#[from] CoreDepositError),
    #[error("SimBootstrapError - TermsTemplate: {0}")]
    TermsTemplate(#[from] TermsTemplateError),
    #[error("SimBootstrapError - Credit: {0}")]
    Credit(#[from] CoreCreditError),
    #[error("SimBootstrapError - Conversion: {0}")]
    Conversion(#[from] ConversionError),
    #[error("SimBootstrapError - Application: {0}")]
    Application(#[from] ApplicationError),
    #[error("SimBootstrapError - ChannelSend")]
    ChannelSend,
}

impl<T> From<SendError<T>> for SimBootstrapError {
    fn from(_: SendError<T>) -> Self {
        SimBootstrapError::ChannelSend
    }
}

impl ErrorSeverity for SimBootstrapError {
    fn severity(&self) -> Level {
        match self {
            Self::User(e) => e.severity(),
            Self::Customer(e) => e.severity(),
            Self::Deposit(e) => e.severity(),
            Self::TermsTemplate(e) => e.severity(),
            Self::Credit(e) => e.severity(),
            Self::Conversion(_) => Level::ERROR,
            Self::Application(e) => e.severity(),
            Self::ChannelSend => Level::ERROR,
        }
    }
}
