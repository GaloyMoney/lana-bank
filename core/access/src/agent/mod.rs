mod entity;
pub mod error;
mod repo;

use std::collections::HashMap;
use tracing::instrument;

use obix::out::{Outbox, OutboxEventMarker};
use tracing_macros::record_error_severity;

use crate::{primitives::*, public::*, publisher::AgentPublisher};

#[cfg(feature = "json-schema")]
pub use entity::AgentEvent;
#[cfg(not(feature = "json-schema"))]
pub(crate) use entity::AgentEvent;
use entity::*;
pub use entity::{Agent, AgentStatus};
pub use error::*;
pub use repo::AgentsSortBy;
pub use repo::agent_cursor;
use repo::*;

/// Return type for agent creation containing the agent entity and the
/// Keycloak client secret (returned once, never stored).
pub struct AgentCreateResult {
    pub agent: Agent,
    pub client_id: String,
    pub client_secret: String,
}

pub struct Agents<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    repo: AgentRepo<E>,
    keycloak: keycloak_client::KeycloakClient,
}

impl<E> Clone for Agents<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            keycloak: self.keycloak.clone(),
        }
    }
}

impl<E> Agents<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        outbox: &Outbox<E>,
        keycloak: keycloak_client::KeycloakClient,
        clock: es_entity::clock::ClockHandle,
    ) -> Self {
        let publisher = AgentPublisher::new(outbox);
        let repo = AgentRepo::new(pool, &publisher, clock);

        Self { repo, keycloak }
    }

    #[record_error_severity]
    #[instrument(name = "core_access.create_agent", skip(self))]
    pub async fn create_agent(
        &self,
        name: impl Into<String> + std::fmt::Debug,
        description: impl Into<String> + std::fmt::Debug,
    ) -> Result<AgentCreateResult, AgentError> {
        let name = name.into();
        let description = description.into();
        let agent_id = AgentId::new();

        // Create confidential client in Keycloak
        let keycloak_result = self
            .keycloak
            .create_agent_client(name.clone(), uuid::Uuid::from(agent_id))
            .await?;

        let new_agent = NewAgent::builder()
            .id(agent_id)
            .name(name)
            .description(description)
            .keycloak_client_id(keycloak_result.client_id.clone())
            .build()
            .expect("Could not build agent");
        let agent = self.repo.create(new_agent).await?;

        Ok(AgentCreateResult {
            agent,
            client_id: keycloak_result.client_id,
            client_secret: keycloak_result.client_secret,
        })
    }

    #[record_error_severity]
    #[instrument(name = "core_access.deactivate_agent", skip(self))]
    pub async fn deactivate_agent(
        &self,
        id: impl Into<AgentId> + std::fmt::Debug,
    ) -> Result<Agent, AgentError> {
        let id = id.into();

        let mut agent = self.repo.find_by_id(id).await?;
        if agent.deactivate().did_execute() {
            // Disable the Keycloak client
            self.keycloak
                .disable_agent_client(&agent.keycloak_client_id)
                .await?;
            self.repo.update(&mut agent).await?;
        }

        Ok(agent)
    }

    #[record_error_severity]
    #[instrument(name = "core_access.find_agent_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        id: impl Into<AgentId> + std::fmt::Debug,
    ) -> Result<Option<Agent>, AgentError> {
        let id = id.into();
        Ok(self.repo.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_access.list_agents", skip(self))]
    pub async fn list_agents(
        &self,
        query: es_entity::PaginatedQueryArgs<agent_cursor::AgentsCursor>,
        sort: es_entity::Sort<AgentsSortBy>,
    ) -> Result<es_entity::PaginatedQueryRet<Agent, agent_cursor::AgentsCursor>, AgentError> {
        Ok(self
            .repo
            .list_for_filters(Default::default(), sort, query)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_access.find_all_agents", skip(self))]
    pub async fn find_all<T: From<Agent>>(
        &self,
        ids: &[AgentId],
    ) -> Result<HashMap<AgentId, T>, AgentError> {
        Ok(self.repo.find_all(ids).await?)
    }

    /// Find an agent by keycloak_client_id — used during authentication
    /// to resolve a Keycloak service-account subject to a lana AgentId.
    pub async fn find_by_keycloak_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<Agent>, AgentError> {
        Ok(self
            .repo
            .maybe_find_by_keycloak_client_id(client_id)
            .await?)
    }
}
