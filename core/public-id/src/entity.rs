use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PublicId")]
pub enum PublicIdEntityEvent {
    Initialized {
        id: PublicId,
        target_id: PublicIdTargetId,
        target_type: IdTargetType,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PublicIdEntity {
    pub id: PublicId,
    pub target_id: PublicIdTargetId,
    pub target_type: IdTargetType,
    events: EntityEvents<PublicIdEntityEvent>,
}

impl core::fmt::Display for PublicIdEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicId: {}", self.id)
    }
}

impl TryFromEvents<PublicIdEntityEvent> for PublicIdEntity {
    fn try_from_events(events: EntityEvents<PublicIdEntityEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PublicIdEntityBuilder::default();

        for event in events.iter_all() {
            match event {
                PublicIdEntityEvent::Initialized {
                    id,
                    target_id,
                    target_type,
                } => {
                    builder = builder
                        .id(id.clone())
                        .target_id(*target_id)
                        .target_type(target_type.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct NewPublicIdEntity {
    #[builder(setter(into))]
    pub(super) id: PublicId,
    #[builder(setter(into))]
    pub(super) target_id: PublicIdTargetId,
    #[builder(setter(into))]
    pub(super) target_type: IdTargetType,
}

impl NewPublicIdEntity {
    pub fn builder() -> NewPublicIdEntityBuilder {
        NewPublicIdEntityBuilder::default()
    }
}

impl IntoEvents<PublicIdEntityEvent> for NewPublicIdEntity {
    fn into_events(self) -> EntityEvents<PublicIdEntityEvent> {
        EntityEvents::init(
            self.id.clone(),
            [PublicIdEntityEvent::Initialized {
                id: self.id,
                target_id: self.target_id,
                target_type: self.target_type,
            }],
        )
    }
}
