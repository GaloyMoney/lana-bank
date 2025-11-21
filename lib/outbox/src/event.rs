use serde::{Deserialize, Serialize, de::DeserializeOwned};

use std::{borrow::Cow, sync::Arc};

es_entity::entity_id! { OutboxEventId }

pub trait OutboxEventMarker<E>:
    serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static + Unpin + From<E>
{
    fn as_event(&self) -> Option<&E>;
}
impl<T> OutboxEventMarker<T> for T
where
    T: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static + Unpin + From<T>,
{
    fn as_event(&self) -> Option<&T> {
        Some(self)
    }
}

pub enum OutboxEvent<P>
where
    P: Serialize + DeserializeOwned + Send,
{
    Persistent(Arc<PersistentOutboxEvent<P>>),
    Ephemeral(Arc<EphemeralOutboxEvent<P>>),
}
impl<P> Clone for OutboxEvent<P>
where
    P: Serialize + DeserializeOwned + Send,
{
    fn clone(&self) -> Self {
        match self {
            Self::Persistent(event) => Self::Persistent(Arc::clone(event)),
            Self::Ephemeral(event) => Self::Ephemeral(Arc::clone(event)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EphemeralEventType(Cow<'static, str>);
impl EphemeralEventType {
    pub const fn new(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }

    pub(crate) fn from_owned(name: String) -> Self {
        Self(Cow::Owned(name))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EphemeralEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct EphemeralOutboxEvent<T>
where
    T: Serialize + DeserializeOwned + Send,
{
    pub event_type: EphemeralEventType,
    pub payload: T,
    pub(crate) tracing_context: Option<tracing_utils::persistence::SerializableTraceContext>,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

impl<T> EphemeralOutboxEvent<T>
where
    T: Serialize + DeserializeOwned + Send,
{
    pub fn inject_trace_parent(&self) {
        if let Some(context) = &self.tracing_context {
            tracing_utils::persistence::set_parent(context);
        }
    }
}

impl<P> From<EphemeralOutboxEvent<P>> for OutboxEvent<P>
where
    P: Serialize + DeserializeOwned + Send,
{
    fn from(event: EphemeralOutboxEvent<P>) -> Self {
        Self::Ephemeral(Arc::new(event))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentOutboxEvent<T>
where
    T: Serialize + DeserializeOwned + Send,
{
    pub id: OutboxEventId,
    pub sequence: EventSequence,
    #[serde(bound = "T: DeserializeOwned")]
    pub payload: Option<T>,
    pub(crate) tracing_context: Option<tracing_utils::persistence::SerializableTraceContext>,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

impl<T> Clone for PersistentOutboxEvent<T>
where
    T: Clone + Serialize + DeserializeOwned + Send,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            sequence: self.sequence,
            payload: self.payload.clone(),
            tracing_context: self.tracing_context.clone(),
            recorded_at: self.recorded_at,
        }
    }
}

impl<P> From<PersistentOutboxEvent<P>> for OutboxEvent<P>
where
    P: Serialize + DeserializeOwned + Send,
{
    fn from(event: PersistentOutboxEvent<P>) -> Self {
        Self::Persistent(Arc::new(event))
    }
}

impl<T> PersistentOutboxEvent<T>
where
    T: Serialize + DeserializeOwned + Send,
{
    pub fn as_event<E>(&self) -> Option<&E>
    where
        T: OutboxEventMarker<E>,
    {
        if let Some(payload) = &self.payload {
            payload.as_event()
        } else {
            None
        }
    }

    pub fn inject_trace_parent(&self) {
        if let Some(context) = &self.tracing_context {
            tracing_utils::persistence::set_parent(context);
        }
    }
}

#[derive(
    sqlx::Type, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, Clone, Serialize, Deserialize,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct EventSequence(i64);
impl EventSequence {
    pub const BEGIN: Self = EventSequence(0);
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl Default for EventSequence {
    fn default() -> Self {
        Self::BEGIN
    }
}

impl From<u64> for EventSequence {
    fn from(n: u64) -> Self {
        Self(n as i64)
    }
}

impl From<EventSequence> for u64 {
    fn from(EventSequence(n): EventSequence) -> Self {
        n as u64
    }
}

impl From<EventSequence> for std::sync::atomic::AtomicU64 {
    fn from(EventSequence(n): EventSequence) -> Self {
        std::sync::atomic::AtomicU64::new(n as u64)
    }
}
impl std::fmt::Display for EventSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
