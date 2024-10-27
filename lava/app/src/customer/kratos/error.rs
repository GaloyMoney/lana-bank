use ory_kratos_client::apis::identity_api::CreateIdentityError as IdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KratosClientError {
    #[error("KratosClientError - KratosError: {0}")]
    CouldNotCreateIdentity(#[from] ory_kratos_client::apis::Error<IdentityError>),
    #[error("KratosClientError - ParseUuidError: {0}")]
    ParseUuidError(#[from] uuid::Error),
}
