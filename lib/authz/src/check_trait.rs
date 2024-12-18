use async_trait::async_trait;

use audit::{AuditInfo, AuditSvc};

use super::error::AuthorizationError;

#[async_trait]
pub trait PermissionCheck: Clone + Sync + Send + 'static {
    type Audit: AuditSvc;

    fn audit(&self) -> &Self::Audit;

    async fn enforce_permission(
        &self,
        sub: &<Self::Audit as AuditSvc>::Subject,
        object: impl Into<<Self::Audit as AuditSvc>::Object> + std::fmt::Debug + Send,
        action: impl Into<<Self::Audit as AuditSvc>::Action> + std::fmt::Debug + Send,
    ) -> Result<AuditInfo, AuthorizationError>;

    async fn evaluate_permission(
        &self,
        sub: &<Self::Audit as AuditSvc>::Subject,
        object: impl Into<<Self::Audit as AuditSvc>::Object> + std::fmt::Debug + Send,
        action: impl Into<<Self::Audit as AuditSvc>::Action> + std::fmt::Debug + Send,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, AuthorizationError>;
}
