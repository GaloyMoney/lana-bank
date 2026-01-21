use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

use audit::{
    AuditCursor, AuditEntry, AuditInfo, AuditSvc, PaginatedQueryArgs, PaginatedQueryRet,
    error::AuditError,
};
use core_access::{CoreAccessAction, CoreAccessEvent, CoreAccessObject, UserId};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

/// A test subject that can be converted from UserId (required by CoreAccess)
#[derive(Debug, Clone, Copy)]
pub struct TestSubject;

impl fmt::Display for TestSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test-subject")
    }
}

impl std::str::FromStr for TestSubject {
    type Err = std::convert::Infallible;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(TestSubject)
    }
}

impl From<UserId> for TestSubject {
    fn from(_: UserId) -> Self {
        TestSubject
    }
}

impl audit::SystemSubject for TestSubject {
    fn system() -> Self {
        TestSubject
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

    async fn record_system_entry_in_tx(
        &self,
        _tx: &mut impl es_entity::AtomicOperation,
        _object: impl Into<Self::Object> + Send,
        _action: impl Into<Self::Action> + Send,
    ) -> Result<AuditInfo, AuditError> {
        Ok(dummy_audit_info())
    }

    async fn record_entry_in_tx(
        &self,
        _tx: &mut impl es_entity::AtomicOperation,
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
    ) -> Result<
        PaginatedQueryRet<AuditEntry<Self::Subject, Self::Object, Self::Action>, AuditCursor>,
        AuditError,
    > {
        unimplemented!("TestAudit::list should not be called")
    }
}

pub mod event {
    use super::*;
    use std::time::Duration;
    use tokio_stream::StreamExt;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreAccess(CoreAccessEvent),
    }

    impl obix::out::OutboxEventMarker<CoreAccessEvent> for DummyEvent {
        fn as_event(&self) -> Option<&CoreAccessEvent> {
            match self {
                Self::CoreAccess(event) => Some(event),
            }
        }
    }

    impl From<CoreAccessEvent> for DummyEvent {
        fn from(event: CoreAccessEvent) -> Self {
            Self::CoreAccess(event)
        }
    }

    /// Executes a use case and waits for a matching event to be published.
    ///
    /// Returns both the use case result and the extracted event payload.
    /// The `matches` predicate receives the use case result to filter events by ID.
    ///
    /// # Arguments
    /// * `outbox` - The outbox to listen for events
    /// * `use_case` - Async closure that executes the operation and returns a result
    /// * `matches` - Predicate that checks if an event matches the result (e.g., by ID)
    ///
    /// # Example
    /// ```ignore
    /// let (user, event) = expect_event(
    ///     &outbox,
    ///     || access.create_user(&TestSubject, &email, role.id),
    ///     |result, e| match e {
    ///         CoreAccessEvent::UserCreated { entity } if entity.id == result.id => {
    ///             Some(entity.clone())
    ///         }
    ///         _ => None,
    ///     },
    /// ).await?;
    /// assert_eq!(event.id, user.id);
    /// ```
    pub async fn expect_event<R, T, F, Fut, E, P>(
        outbox: &obix::Outbox<DummyEvent>,
        use_case: F,
        matches: P,
    ) -> anyhow::Result<(R, T)>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<R, E>>,
        E: std::fmt::Debug,
        P: Fn(&R, &CoreAccessEvent) -> Option<T>,
    {
        let mut listener = outbox.listen_persisted(None);

        // Execute the use case
        let result = use_case().await.expect("use case should succeed");

        // Wait for the matching event (filtered by result ID)
        let event = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let event = listener.next().await.expect("should receive an event");
                if let Some(extracted) =
                    event.as_event::<CoreAccessEvent>().and_then(|e| matches(&result, e))
                {
                    return extracted;
                }
            }
        })
        .await?;

        Ok((result, event))
    }
}
