use es_entity::*;
use outbox::OutboxEventMarker;
use sqlx::PgPool;

use crate::{
    entity::{DomainConfiguration, DomainConfigurationEvent},
    error::DomainConfigurationError,
    primitives::DomainConfigurationKey,
    publisher::DomainConfigurationPublisher,
};

#[derive(EsRepo)]
#[es_repo(
    entity = "DomainConfiguration",
    id = "DomainConfigurationKey",
    err = "DomainConfigurationError",
    columns(
        value(ty = "serde_json::Value"),
        updated_by(ty = "String"),
        updated_at(ty = "chrono::DateTime<chrono::Utc>"),
        reason(ty = "Option<String>"),
        correlation_id(ty = "Option<String>"),
    ),
    tbl = "core_domain_configurations",
    events_tbl = "core_domain_configuration_events",
    post_persist_hook = "publish"
)]
pub struct DomainConfigurationRepo<E>
where
    E: OutboxEventMarker<DomainConfigurationEvent>,
{
    pool: PgPool,
    publisher: DomainConfigurationPublisher<E>,
}

impl<E> Clone for DomainConfigurationRepo<E>
where
    E: OutboxEventMarker<DomainConfigurationEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}

impl<E> DomainConfigurationRepo<E>
where
    E: OutboxEventMarker<DomainConfigurationEvent>,
{
    pub fn new(pool: &PgPool, publisher: &DomainConfigurationPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &DomainConfiguration,
        new_events: es_entity::LastPersisted<'_, DomainConfigurationEvent>,
    ) -> Result<(), DomainConfigurationError> {
        self.publisher.publish(op, entity, new_events).await
    }
}
