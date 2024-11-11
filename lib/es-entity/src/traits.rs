use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use super::{error::EsEntityError, events::EntityEvents};

#[async_trait]
pub trait DatabaseOperation<'t> {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
    fn tx(&'t mut self) -> &mut sqlx::Transaction<'t, sqlx::Postgres>;
    async fn commit(self) -> Result<(), sqlx::Error>;
}

pub trait EsEvent: DeserializeOwned + Serialize + Send + Sync {
    type EntityId: Clone + PartialEq + sqlx::Type<sqlx::Postgres> + std::hash::Hash + Send + Sync;
}

pub trait IntoEvents<E: EsEvent> {
    fn into_events(self) -> EntityEvents<E>;
}

pub trait TryFromEvents<E: EsEvent> {
    fn try_from_events(events: EntityEvents<E>) -> Result<Self, EsEntityError>
    where
        Self: Sized;
}

pub trait EsEntity<E: EsEvent>: TryFromEvents<E> {
    fn events_mut(&mut self) -> &mut EntityEvents<E>;
    fn events(&self) -> &EntityEvents<E>;
}

pub trait RetryableInto<T>: Into<T> + Copy + std::fmt::Debug {}
impl<T, O> RetryableInto<O> for T where T: Into<O> + Copy + std::fmt::Debug {}
