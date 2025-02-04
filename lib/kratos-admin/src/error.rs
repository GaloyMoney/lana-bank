use ory_kratos_client::apis::identity_api::CreateIdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KratosAdminError {
    #[error("KratosAdminError - OryKratosAdminApiCreateIdentityError: {0}")]
    KratosAdminApiCreateIdentityError(#[from] ory_kratos_client::apis::Error<CreateIdentityError>),
    #[error("KratosAdminError - UuidError: {0}")]
    Uuid(#[from] ::uuid::Error),
}
