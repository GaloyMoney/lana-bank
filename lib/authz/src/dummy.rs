use async_trait::async_trait;

use std::fmt;

use audit::{
    AuditCursor, AuditEntry, AuditInfo, AuditSvc, PaginatedQueryArgs, PaginatedQueryRet,
    error::AuditError,
};

use crate::{PermissionCheck, error::AuthorizationError};

#[derive(Clone)]
pub struct DummyAudit<A, O> {
    _phantom: std::marker::PhantomData<(A, O)>,
}
#[derive(Clone)]
pub struct DummyPerms<A, O> {
    audit: DummyAudit<A, O>,
}
impl<A, O> DummyPerms<A, O> {
    pub fn new() -> Self {
        Self {
            audit: DummyAudit {
                _phantom: std::marker::PhantomData,
            },
        }
    }
}

impl<A, O> Default for DummyPerms<A, O> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DummySubject;

impl audit::SystemSubject for DummySubject {
    fn system(_actor: audit::SystemActor) -> Self {
        DummySubject
    }
}

impl fmt::Display for DummySubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "system:unknown")
    }
}

impl std::str::FromStr for DummySubject {
    type Err = std::convert::Infallible;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(DummySubject)
    }
}

#[async_trait]
impl<A, O> AuditSvc for DummyAudit<A, O>
where
    A: std::str::FromStr + std::fmt::Display + std::fmt::Debug + Copy + Send + Sync + 'static,
    O: std::str::FromStr + std::fmt::Display + std::fmt::Debug + Copy + Send + Sync + 'static,
{
    type Subject = DummySubject;
    type Object = O;
    type Action = A;

    fn pool(&self) -> &sqlx::PgPool {
        unimplemented!()
    }

    async fn record_system_entry(
        &self,
        _actor: audit::SystemActor,
        _object: impl Into<Self::Object> + Send,
        _action: impl Into<Self::Action> + Send,
    ) -> Result<AuditInfo, AuditError> {
        Ok(dummy_audit_info())
    }

    async fn record_entry(
        &self,
        _subject: &Self::Subject,
        _object: impl Into<Self::Object> + Send,
        _action: impl Into<Self::Action> + Send,
        _authorized: bool,
    ) -> Result<AuditInfo, AuditError> {
        Ok(dummy_audit_info())
    }

    async fn record_system_entry_in_op(
        &self,
        _op: &mut impl es_entity::AtomicOperation,
        _actor: audit::SystemActor,
        _object: impl Into<Self::Object> + Send,
        _action: impl Into<Self::Action> + Send,
    ) -> Result<AuditInfo, AuditError> {
        Ok(dummy_audit_info())
    }

    async fn record_entry_in_op(
        &self,
        _op: &mut impl es_entity::AtomicOperation,
        _subject: &Self::Subject,
        _object: impl Into<Self::Object> + Send,
        _action: impl Into<Self::Action> + Send,
        _authorized: bool,
    ) -> Result<AuditInfo, AuditError> {
        Ok(dummy_audit_info())
    }

    async fn list(
        &self,
        _query: PaginatedQueryArgs<AuditCursor>,
        _subject_filter: Option<String>,
        _authorized_filter: Option<bool>,
        _object_filter: Option<String>,
        _action_filter: Option<String>,
    ) -> Result<
        PaginatedQueryRet<AuditEntry<Self::Subject, Self::Object, Self::Action>, AuditCursor>,
        AuditError,
    > {
        // This method should never be called on the dummy implementation
        // as it's only used for testing authorization logic, not audit querying
        unimplemented!("DummyAudit::list should not be called - this is a test-only implementation")
    }

    async fn list_subjects(&self) -> Result<Vec<String>, AuditError> {
        unimplemented!("DummyAudit::list_subjects - test-only implementation")
    }
}

fn dummy_audit_info() -> audit::AuditInfo {
    AuditInfo {
        audit_entry_id: audit::AuditEntryId::from(1),
        sub: "sub".to_string(),
    }
}

#[async_trait]
impl<A, O> PermissionCheck for DummyPerms<A, O>
where
    A: std::str::FromStr
        + std::fmt::Display
        + std::fmt::Debug
        + Copy
        + Clone
        + Send
        + Sync
        + 'static,
    O: std::str::FromStr
        + std::fmt::Display
        + std::fmt::Debug
        + Copy
        + Clone
        + Send
        + Sync
        + 'static,
{
    type Audit = DummyAudit<A, O>;

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
