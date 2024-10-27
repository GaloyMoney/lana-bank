use sqlx_adapter::casbin::error::Error as CasbinError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthorizationError {
    #[error("AuthorizationError - CasbinError: {0}")]
    Casbin(CasbinError),
    #[error("AuthorizationError - NotAuthorized")]
    NotAuthorized,
    #[error("AuthorizationError - PermissionAlreadyExistsForRole: {0}")]
    PermissionAlreadyExistsForRole(String),
    #[error("AuthorizationError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("AuthorizationError - RoleParseError: Could not parse '{0}'")]
    RoleParseError(String),
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
                    return AuthorizationError::PermissionAlreadyExistsForRole(
                        db_error.message().to_string(),
                    );
                }
            }
        }
        AuthorizationError::Casbin(error)
    }
}
