use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::sync::Arc;

es_entity::entity_id! { OutboxEventId }

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for OutboxEventId {
    fn schema_name() -> String {
        "OutboxEventId".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }
}

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
}
impl<P> Clone for OutboxEvent<P>
where
    P: Serialize + DeserializeOwned + Send,
{
    fn clone(&self) -> Self {
        match self {
            Self::Persistent(event) => Self::Persistent(Arc::clone(event)),
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PersistentOutboxEvent<T>
where
    T: Serialize + DeserializeOwned + Send,
{
    pub id: OutboxEventId,
    pub sequence: EventSequence,
    #[serde(bound = "T: DeserializeOwned")]
    pub payload: Option<T>,
    #[cfg_attr(feature = "schemars", schemars(skip))]
    pub(crate) tracing_context: Option<tracing_utils::persistence::SerializableTraceContext>,
    #[cfg_attr(feature = "schemars", schemars(skip))]
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
