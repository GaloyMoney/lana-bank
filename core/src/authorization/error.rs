use sqlx_adapter::casbin::error::Error as CasbinError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthorizationError {
    #[error("AuthorizationError - CasbinError: {0}")]
    Casbin(CasbinError),
    #[error("AuthorizationError - NotAuthorized")]
    NotAuthorized,
    #[error("AuthorizationError - DuplicateRule: {0}")]
    DuplicateRule(String),
}

impl From<CasbinError> for AuthorizationError {
    fn from(error: CasbinError) -> Self {
        if let CasbinError::AdapterError(adapter_error) = &error {
            if let Some(sqlx::Error::Database(db_error)) = adapter_error
                .0
                .source()
                .and_then(|e| e.downcast_ref::<sqlx::Error>())
            {
                if db_error.code() == Some("23505".into()) {
                    return AuthorizationError::DuplicateRule(db_error.message().to_string());
                }
            }
        }
        AuthorizationError::Casbin(error)
    }
}
