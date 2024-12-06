use async_trait::async_trait;

use std::fmt;

use audit::{AuditInfo, AuditSvc};

use crate::{error::AuthorizationError, PermissionCheck};

#[derive(Clone)]
pub struct DummyAudit;
#[derive(Clone)]
pub struct DummyPerms {
    audit: DummyAudit,
}
#[derive(Debug, Clone, Copy)]
pub struct DummyItem;
impl audit::SystemSubject for DummyItem {
    fn system() -> Self {
        DummyItem
    }
}

impl fmt::Display for DummyItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "item")
    }
}

impl std::str::FromStr for DummyItem {
    type Err = std::convert::Infallible;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(DummyItem)
    }
}

#[async_trait]
impl AuditSvc for DummyAudit {
    type Subject = DummyItem;
    type Object = DummyItem;
    type Action = DummyItem;

    fn pool(&self) -> &sqlx::PgPool {
        unimplemented!()
    }

    async fn record_entry(
        &self,
        _subject: &Self::Subject,
        _object: impl Into<Self::Object> + Send,
        _action: impl Into<Self::Action> + Send,
        _authorized: bool,
    ) -> Result<AuditInfo, audit::error::AuditError> {
        Ok(dummy_audit_info())
    }
}

fn dummy_audit_info() -> audit::AuditInfo {
    AuditInfo {
        audit_entry_id: audit::AuditEntryId::from(1),
        sub: "sub".to_string(),
    }
}

#[async_trait]
impl PermissionCheck for DummyPerms {
    type Audit = DummyAudit;

    fn audit(&self) -> &Self::Audit {
        &self.audit
    }

    async fn enforce_permission(
        &self,
        _sub: &<Self::Audit as AuditSvc>::Subject,
        _object: impl Into<<Self::Audit as AuditSvc>::Object> + std::fmt::Debug + Send,
        _action: impl Into<<Self::Audit as AuditSvc>::Action> + std::fmt::Debug + Send,
    ) -> Result<AuditInfo, AuthorizationError> {
        Ok(dummy_audit_info())
    }

    async fn evaluate_permission(
        &self,
        _sub: &<Self::Audit as AuditSvc>::Subject,
        _object: impl Into<<Self::Audit as AuditSvc>::Object> + std::fmt::Debug + Send,
        _action: impl Into<<Self::Audit as AuditSvc>::Action> + std::fmt::Debug + Send,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, AuthorizationError> {
        if enforce {
            Ok(Some(dummy_audit_info()))
        } else {
            Ok(None)
        }
    }
}
