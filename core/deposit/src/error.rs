use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum CoreDepositError {
    #[error("CoreDepositError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("CoreDepositError - AuditError: {0}")]
    AuditError(#[from] audit::error::AuditError),
    #[error("CoreDepositError - AuthorizationError: {0}")]
    AuthorizationError(#[from] authz::error::AuthorizationError),
    #[error("CoreDepositError - DepositAccountError: {0}")]
    DepositAccountError(#[from] crate::account::error::DepositAccountError),
    #[error("CoreDepositError - DepositError: {0}")]
    DepositError(#[from] crate::deposit::error::DepositError),
    #[error("CoreDepositError - WithdrawalError: {0}")]
    WithdrawalError(#[from] crate::withdrawal::error::WithdrawalError),
    #[error("CoreDepositError - DepositLedgerError: {0}")]
    DepositLedgerError(#[from] crate::ledger::error::DepositLedgerError),
    #[error("CoreDepositError - GovernanceError: {0}")]
    GovernanceError(#[from] governance::error::GovernanceError),
    #[error("CoreDepositError - CustomerError: {0}")]
    CustomerError(#[from] core_customer::error::CustomerError),
    #[error("CoreDepositError - ChartLookupError: {0}")]
    ChartLookupError(#[from] chart_primitives::ChartLookupError),
    #[error("CoreDepositError - JobError: {0}")]
    JobError(#[from] job::error::JobError),
    #[error("CoreDepositError - RegisterEventHandler: {0}")]
    RegisterEventHandler(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("CoreDepositError - ProcessError: {0}")]
    ProcessError(#[from] crate::processes::error::ProcessError),
    #[error("CoreDepositError - SubjectIsNotDepositAccountHolder")]
    SubjectIsNotDepositAccountHolder,
    #[error("CoreDepositError - DepositAccountNotFound")]
    DepositAccountNotFound,
    #[error("CoreDepositError - ChartIdMismatch")]
    ChartIdMismatch,
    #[error("CoreDepositError - DepositConfigAlreadyExists")]
    DepositConfigAlreadyExists,
    #[error("CoreDepositError - DepositAccountInactive")]
    DepositAccountInactive,
    #[error("CoreDepositError - DepositAccountFrozen")]
    DepositAccountFrozen,
    #[error("CoreDepositError - DepositAccountClosed")]
    DepositAccountClosed,
    #[error("CoreDepositError - WithdrawalBuilderError: {0}")]
    WithdrawalBuilderError(#[from] super::NewWithdrawalBuilderError),
    #[error("CoreDepositError - DepositBuilderError: {0}")]
    DepositBuilderError(#[from] super::NewDepositBuilderError),
    #[error("CoreDepositError - PublicIdError: {0}")]
    PublicIdError(#[from] public_id::PublicIdError),
    #[error("CoreDepositError - DomainConfigError: {0}")]
    DomainConfigError(#[from] domain_config::DomainConfigError),
    #[error("CoreDepositError - CustomerNotVerified")]
    CustomerNotVerified,
}

// Two-hop From impls: repo typed errors -> sub-module error -> CoreDepositError

impl From<crate::account::DepositAccountCreateError> for CoreDepositError {
    fn from(e: crate::account::DepositAccountCreateError) -> Self {
        Self::DepositAccountError(e.into())
    }
}

impl From<crate::account::DepositAccountFindError> for CoreDepositError {
    fn from(e: crate::account::DepositAccountFindError) -> Self {
        Self::DepositAccountError(e.into())
    }
}

impl From<crate::account::DepositAccountModifyError> for CoreDepositError {
    fn from(e: crate::account::DepositAccountModifyError) -> Self {
        Self::DepositAccountError(e.into())
    }
}

impl From<crate::account::DepositAccountQueryError> for CoreDepositError {
    fn from(e: crate::account::DepositAccountQueryError) -> Self {
        Self::DepositAccountError(e.into())
    }
}

impl From<crate::deposit::DepositCreateError> for CoreDepositError {
    fn from(e: crate::deposit::DepositCreateError) -> Self {
        Self::DepositError(e.into())
    }
}

impl From<crate::deposit::DepositFindError> for CoreDepositError {
    fn from(e: crate::deposit::DepositFindError) -> Self {
        Self::DepositError(e.into())
    }
}

impl From<crate::deposit::DepositModifyError> for CoreDepositError {
    fn from(e: crate::deposit::DepositModifyError) -> Self {
        Self::DepositError(e.into())
    }
}

impl From<crate::deposit::DepositQueryError> for CoreDepositError {
    fn from(e: crate::deposit::DepositQueryError) -> Self {
        Self::DepositError(e.into())
    }
}

impl From<crate::withdrawal::WithdrawalCreateError> for CoreDepositError {
    fn from(e: crate::withdrawal::WithdrawalCreateError) -> Self {
        Self::WithdrawalError(e.into())
    }
}

impl From<crate::withdrawal::WithdrawalFindError> for CoreDepositError {
    fn from(e: crate::withdrawal::WithdrawalFindError) -> Self {
        Self::WithdrawalError(e.into())
    }
}

impl From<crate::withdrawal::WithdrawalModifyError> for CoreDepositError {
    fn from(e: crate::withdrawal::WithdrawalModifyError) -> Self {
        Self::WithdrawalError(e.into())
    }
}

impl From<crate::withdrawal::WithdrawalQueryError> for CoreDepositError {
    fn from(e: crate::withdrawal::WithdrawalQueryError) -> Self {
        Self::WithdrawalError(e.into())
    }
}

impl CoreDepositError {
    pub fn is_account_already_exists(&self) -> bool {
        matches!(
            self,
            Self::DepositLedgerError(crate::ledger::error::DepositLedgerError::CalaAccount(
                cala_ledger::account::error::AccountError::ExternalIdAlreadyExists(_)
            ))
        )
    }
}

impl ErrorSeverity for CoreDepositError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::AuditError(e) => e.severity(),
            Self::AuthorizationError(e) => e.severity(),
            Self::DepositAccountError(e) => e.severity(),
            Self::DepositError(e) => e.severity(),
            Self::WithdrawalError(e) => e.severity(),
            Self::DepositLedgerError(e) => e.severity(),
            Self::GovernanceError(e) => e.severity(),
            Self::CustomerError(e) => e.severity(),
            Self::ChartLookupError(e) => e.severity(),
            Self::JobError(_) => Level::ERROR,
            Self::RegisterEventHandler(_) => Level::ERROR,
            Self::ProcessError(e) => e.severity(),
            Self::SubjectIsNotDepositAccountHolder => Level::WARN,
            Self::DepositAccountNotFound => Level::WARN,
            Self::ChartIdMismatch => Level::ERROR,
            Self::DepositConfigAlreadyExists => Level::WARN,
            Self::DepositAccountInactive => Level::WARN,
            Self::DepositAccountFrozen => Level::WARN,
            Self::DepositAccountClosed => Level::WARN,
            Self::WithdrawalBuilderError(_) => Level::ERROR,
            Self::DepositBuilderError(_) => Level::ERROR,
            Self::PublicIdError(e) => e.severity(),
            Self::DomainConfigError(e) => e.severity(),
            Self::CustomerNotVerified => Level::WARN,
        }
    }
}
