use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::{primitives::*, public::CoreAccessEvent, publisher::AgentPublisher};

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "Agent",
    columns(
        name(ty = "String", list_by),
        keycloak_client_id(ty = "String", list_by),
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub(crate) struct AgentRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[allow(dead_code)]
    pool: PgPool,
    publisher: AgentPublisher<E>,
    clock: ClockHandle,
}

impl<E> AgentRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(pool: &PgPool, publisher: &AgentPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
        }
    }

    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Agent,
        new_events: es_entity::LastPersisted<'_, AgentEvent>,
    ) -> Result<(), sqlx::Error> {
        self.publisher
            .publish_agent_in_op(op, entity, new_events)
            .await
    }
}

impl<E> Clone for AgentRepo<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl From<(AgentsSortBy, &Agent)> for agent_cursor::AgentsCursor {
    fn from(agent_with_sort: (AgentsSortBy, &Agent)) -> Self {
        let (sort, agent) = agent_with_sort;
        match sort {
            AgentsSortBy::CreatedAt => agent_cursor::AgentsByCreatedAtCursor::from(agent).into(),
            AgentsSortBy::Id => agent_cursor::AgentsByIdCursor::from(agent).into(),
            AgentsSortBy::Name => agent_cursor::AgentsByNameCursor::from(agent).into(),
            AgentsSortBy::KeycloakClientId => {
                agent_cursor::AgentsByKeycloakClientIdCursor::from(agent).into()
            }
        }
    }
}
