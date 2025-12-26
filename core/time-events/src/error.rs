use domain_config::DomainConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeEventsError {
    #[error("DomainConfigError: {0}")]
    DomainConfig(#[from] DomainConfigError),
}
