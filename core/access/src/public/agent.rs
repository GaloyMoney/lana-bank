use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{agent::Agent, primitives::AgentId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicAgent {
    pub id: AgentId,
    pub name: String,
    pub keycloak_client_id: String,
}

impl From<&Agent> for PublicAgent {
    fn from(entity: &Agent) -> Self {
        PublicAgent {
            id: entity.id,
            name: entity.name.clone(),
            keycloak_client_id: entity.keycloak_client_id.clone(),
        }
    }
}
