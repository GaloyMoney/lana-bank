use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

use crate::primitives::PermissionSetId;

use super::permission_set::error::{
    PermissionSetCreateError, PermissionSetFindError, PermissionSetModifyError,
    PermissionSetQueryError,
};
use super::role::error::{RoleCreateError, RoleFindError, RoleModifyError, RoleQueryError};

#[derive(Error, Debug)]
pub enum CoreAccessError {
    #[error("CoreAccessError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreAccessError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreAccessError - UserError: {0}")]
    UserError(#[from] super::user::UserError),
    #[error("CoreAccessError - RoleError: {0}")]
    RoleError(#[from] super::role::RoleError),
    #[error("CoreAccessError - PermissionSetError: {0}")]
    PermissionSetError(#[from] super::permission_set::PermissionSetError),
    #[error("CoreAccessError - PermissionSetNotFound: {0}")]
    PermissionSetNotFound(PermissionSetId),
}

impl From<RoleCreateError> for CoreAccessError {
    fn from(e: RoleCreateError) -> Self {
        Self::RoleError(e.into())
    }
}

impl From<RoleFindError> for CoreAccessError {
    fn from(e: RoleFindError) -> Self {
        Self::RoleError(e.into())
    }
}

impl From<RoleModifyError> for CoreAccessError {
    fn from(e: RoleModifyError) -> Self {
        Self::RoleError(e.into())
    }
}

impl From<RoleQueryError> for CoreAccessError {
    fn from(e: RoleQueryError) -> Self {
        Self::RoleError(e.into())
    }
}

impl From<PermissionSetCreateError> for CoreAccessError {
    fn from(e: PermissionSetCreateError) -> Self {
        Self::PermissionSetError(e.into())
    }
}

impl From<PermissionSetFindError> for CoreAccessError {
    fn from(e: PermissionSetFindError) -> Self {
        Self::PermissionSetError(e.into())
    }
}

impl From<PermissionSetModifyError> for CoreAccessError {
    fn from(e: PermissionSetModifyError) -> Self {
        Self::PermissionSetError(e.into())
    }
}

impl From<PermissionSetQueryError> for CoreAccessError {
    fn from(e: PermissionSetQueryError) -> Self {
        Self::PermissionSetError(e.into())
    }
}

impl ErrorSeverity for CoreAccessError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuthorizationError(e) => e.severity(),
            Self::UserError(e) => e.severity(),
            Self::RoleError(e) => e.severity(),
            Self::PermissionSetError(e) => e.severity(),
            Self::PermissionSetNotFound(_) => Level::WARN,
        }
    }
}
