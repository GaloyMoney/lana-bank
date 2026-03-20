use async_graphql::*;
use es_entity::Sort;

use crate::graphql::primitives::SortDirection;
use crate::primitives::*;

use lana_app::access::{
    Agent as DomainAgent, AgentStatus as DomainAgentStatus,
    agent::AgentsSortBy as DomainAgentsSortBy,
};

#[derive(SimpleObject, Clone)]
pub struct Agent {
    id: ID,
    agent_id: UUID,
    name: String,
    description: String,
    status: AgentStatus,
    keycloak_client_id: String,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainAgent>,
}

impl From<DomainAgent> for Agent {
    fn from(agent: DomainAgent) -> Self {
        Self {
            id: agent.id.to_global_id(),
            agent_id: UUID::from(agent.id),
            name: agent.name.clone(),
            description: agent.description.clone(),
            status: AgentStatus::from(agent.status),
            keycloak_client_id: agent.keycloak_client_id.clone(),
            created_at: agent.created_at().into(),
            entity: Arc::new(agent),
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum AgentStatus {
    Active,
    Inactive,
}

impl From<DomainAgentStatus> for AgentStatus {
    fn from(status: DomainAgentStatus) -> Self {
        match status {
            DomainAgentStatus::Active => AgentStatus::Active,
            DomainAgentStatus::Inactive => AgentStatus::Inactive,
        }
    }
}

#[derive(InputObject)]
pub struct AgentCreateInput {
    pub name: String,
    pub description: String,
}

#[derive(SimpleObject)]
pub struct AgentCreatePayload {
    pub agent: Agent,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(InputObject)]
pub struct AgentDeactivateInput {
    pub agent_id: UUID,
}

crate::mutation_payload! { AgentDeactivatePayload, agent: Agent }

#[derive(Enum, Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum AgentSortBy {
    #[default]
    CreatedAt,
    Id,
    Name,
}

impl From<AgentSortBy> for DomainAgentsSortBy {
    fn from(sort: AgentSortBy) -> Self {
        match sort {
            AgentSortBy::CreatedAt => DomainAgentsSortBy::CreatedAt,
            AgentSortBy::Id => DomainAgentsSortBy::Id,
            AgentSortBy::Name => DomainAgentsSortBy::Name,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct AgentsSort {
    #[graphql(default)]
    pub by: AgentSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<AgentsSort> for Sort<DomainAgentsSortBy> {
    fn from(sort: AgentsSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<AgentsSort> for DomainAgentsSortBy {
    fn from(sort: AgentsSort) -> Self {
        sort.by.into()
    }
}
