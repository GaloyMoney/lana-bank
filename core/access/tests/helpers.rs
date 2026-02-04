use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

use audit::{
    AuditCursor, AuditEntry, AuditInfo, AuditSvc, PaginatedQueryArgs, PaginatedQueryRet,
    error::AuditError,
};
use core_access::{
    AuthRoleToken, CoreAccess, CoreAccessAction, CoreAccessEvent, CoreAccessObject, UserId,
    config::AccessConfig,
};

pub const ROLE_NAME_ADMIN: &str = "admin";
pub const ROLE_NAME_BANK_MANAGER: &str = "bank-manager";
pub const ROLE_NAME_ACCOUNTANT: &str = "accountant";

pub const PREDEFINED_ROLES: &[(&str, &[&str])] = &[
    (
        ROLE_NAME_ADMIN,
        &[
            core_access::PERMISSION_SET_ACCESS_VIEWER,
            core_access::PERMISSION_SET_ACCESS_WRITER,
        ],
    ),
    (
        ROLE_NAME_BANK_MANAGER,
        &[core_access::PERMISSION_SET_ACCESS_VIEWER],
    ),
    (
        ROLE_NAME_ACCOUNTANT,
        &[core_access::PERMISSION_SET_ACCESS_VIEWER],
    ),
];

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

/// A test subject that wraps a unique UserId, so Casbin can distinguish subjects.
#[derive(Debug, Clone, Copy)]
pub struct TestSubject(UserId);

impl Default for TestSubject {
    fn default() -> Self {
        TestSubject(UserId::new())
    }
}

impl TestSubject {
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for TestSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test-subject:{}", self.0)
    }
}

impl std::str::FromStr for TestSubject {
    type Err = std::convert::Infallible;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(TestSubject::new())
    }
}

impl From<UserId> for TestSubject {
    fn from(id: UserId) -> Self {
        TestSubject(id)
    }
}

impl audit::SystemSubject for TestSubject {
    fn system() -> Self {
        TestSubject(UserId::new())
    }
}

/// A test audit implementation that satisfies CoreAccess requirements
#[derive(Clone)]
pub struct TestAudit;

fn dummy_audit_info() -> AuditInfo {
    AuditInfo {
        audit_entry_id: audit::AuditEntryId::from(1),
        sub: "test-subject".to_string(),
    }
}

#[async_trait]
impl AuditSvc for TestAudit {
    type Subject = TestSubject;
    type Object = CoreAccessObject;
    type Action = CoreAccessAction;

    fn pool(&self) -> &sqlx::PgPool {
        unimplemented!()
    }

    async fn record_system_entry(
        &self,
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
        unimplemented!("TestAudit::list should not be called")
    }

    async fn list_subjects(&self) -> Result<Vec<String>, AuditError> {
        unimplemented!("TestAudit::list_subjects should not be called")
    }
}

pub mod event {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreAccess(CoreAccessEvent),
        #[serde(other)]
        Unknown,
    }

    pub use obix::test_utils::expect_event;
}

pub async fn init_access(
    pool: &sqlx::PgPool,
    clock: es_entity::clock::ClockHandle,
) -> anyhow::Result<(
    CoreAccess<TestAudit, event::DummyEvent>,
    TestSubject,
    obix::Outbox<event::DummyEvent>,
)> {
    let superuser_email = "superuser@test.io".to_string();
    let outbox = obix::Outbox::<event::DummyEvent>::init(
        pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let audit = TestAudit;
    let authz = authz::Authorization::<TestAudit, AuthRoleToken>::init(pool, &audit).await?;

    let config = AccessConfig {
        superuser_email: Some(superuser_email.clone()),
    };

    let access = CoreAccess::init(
        pool,
        config,
        CoreAccessAction::actions(),
        PREDEFINED_ROLES,
        &authz,
        &outbox,
        clock,
    )
    .await?;

    let superuser = access
        .users()
        .find_by_email(None, &superuser_email)
        .await?
        .expect("Superuser not found");

    Ok((access, TestSubject::from(superuser.id), outbox))
}
