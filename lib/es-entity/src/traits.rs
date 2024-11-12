use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use super::{error::EsEntityError, events::EntityEvents, nested::*};

pub trait EsEvent: DeserializeOwned + Serialize + Send + Sync {
    type EntityId: Clone
        + PartialEq
        + sqlx::Type<sqlx::Postgres>
        + Eq
        + std::hash::Hash
        + Send
        + Sync;
}

pub trait IntoEvents<E: EsEvent> {
    fn into_events(self) -> EntityEvents<E>;
}

pub trait TryFromEvents<E: EsEvent> {
    fn try_from_events(events: EntityEvents<E>) -> Result<Self, EsEntityError>
    where
        Self: Sized;
}

pub trait EsEntity: TryFromEvents<Self::Event> {
    type Event: EsEvent;
    type New: IntoEvents<Self::Event>;

    fn events(&self) -> &EntityEvents<Self::Event>;
    fn events_mut(&mut self) -> &mut EntityEvents<Self::Event>;
}

pub trait Parent<T: EsEntity> {
    fn nested(&self) -> &Nested<T>;
    fn nested_mut(&mut self) -> &mut Nested<T>;
}

pub trait EsRepo {
    type Entity: EsEntity;
}

#[async_trait]
pub trait PopluateNested<C>: EsRepo {
    type Err;

    async fn popluate(
        &self,
        lookup: std::collections::HashMap<C, &mut Nested<<Self as EsRepo>::Entity>>,
    ) -> Result<(), Self::Err>;
}

pub trait RetryableInto<T>: Into<T> + Copy + std::fmt::Debug {}
impl<T, O> RetryableInto<O> for T where T: Into<O> + Copy + std::fmt::Debug {}
