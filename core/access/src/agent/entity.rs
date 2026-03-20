use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString,
)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AgentStatus {
    Active,
    Inactive,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AgentId")]
pub enum AgentEvent {
    Initialized {
        id: AgentId,
        name: String,
        description: String,
        keycloak_client_id: String,
    },
    Deactivated,
    Reactivated,
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub description: String,
    pub keycloak_client_id: String,
    pub status: AgentStatus,
    events: EntityEvents<AgentEvent>,
}

impl Agent {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub(crate) fn deactivate(&mut self) -> Idempotent<()> {
        if self.status == AgentStatus::Inactive {
            return Idempotent::AlreadyApplied;
        }
        self.events.push(AgentEvent::Deactivated);
        self.status = AgentStatus::Inactive;
        Idempotent::Executed(())
    }

    pub(crate) fn reactivate(&mut self) -> Idempotent<()> {
        if self.status == AgentStatus::Active {
            return Idempotent::AlreadyApplied;
        }
        self.events.push(AgentEvent::Reactivated);
        self.status = AgentStatus::Active;
        Idempotent::Executed(())
    }
}

impl core::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Agent: {}, name: {}", self.id, self.name)
    }
}

impl TryFromEvents<AgentEvent> for Agent {
    fn try_from_events(events: EntityEvents<AgentEvent>) -> Result<Self, EntityHydrationError> {
        let mut builder = AgentBuilder::default();
        let mut status = AgentStatus::Active;

        for event in events.iter_all() {
            match event {
                AgentEvent::Initialized {
                    id,
                    name,
                    description,
                    keycloak_client_id,
                } => {
                    builder = builder
                        .id(*id)
                        .name(name.clone())
                        .description(description.clone())
                        .keycloak_client_id(keycloak_client_id.clone());
                }
                AgentEvent::Deactivated => {
                    status = AgentStatus::Inactive;
                }
                AgentEvent::Reactivated => {
                    status = AgentStatus::Active;
                }
            }
        }

        builder.status(status).events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewAgent {
    #[builder(setter(into))]
    pub(super) id: AgentId,
    #[builder(setter(into))]
    pub(super) name: String,
    #[builder(setter(into))]
    pub(super) description: String,
    #[builder(setter(into))]
    pub(super) keycloak_client_id: String,
}

impl NewAgent {
    pub fn builder() -> NewAgentBuilder {
        let id = AgentId::new();
        let mut builder = NewAgentBuilder::default();
        builder.id(id);
        builder
    }
}

impl IntoEvents<AgentEvent> for NewAgent {
    fn into_events(self) -> EntityEvents<AgentEvent> {
        EntityEvents::init(
            self.id,
            [AgentEvent::Initialized {
                id: self.id,
                name: self.name,
                description: self.description,
                keycloak_client_id: self.keycloak_client_id,
            }],
        )
    }
}
