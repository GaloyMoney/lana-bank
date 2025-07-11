use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "Id")]
pub enum PublicIdEvent {
    Initialized {
        id: Id,
        target_id: PublicIdTargetId,
        target_type: IdTargetType,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PublicId {
    pub id: Id,
    pub target_id: PublicIdTargetId,
    pub target_type: IdTargetType,
    events: EntityEvents<PublicIdEvent>,
}

impl core::fmt::Display for PublicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicId: {}", self.id)
    }
}

impl TryFromEvents<PublicIdEvent> for PublicId {
    fn try_from_events(events: EntityEvents<PublicIdEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PublicIdBuilder::default();

        for event in events.iter_all() {
            match event {
                PublicIdEvent::Initialized {
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
pub struct NewPublicId {
    #[builder(setter(into))]
    pub(super) id: Id,
    #[builder(setter(into))]
    pub(super) target_id: PublicIdTargetId,
    #[builder(setter(into))]
    pub(super) target_type: IdTargetType,
}

impl NewPublicId {
    pub fn builder() -> NewPublicIdBuilder {
        NewPublicIdBuilder::default()
    }
}

impl IntoEvents<PublicIdEvent> for NewPublicId {
    fn into_events(self) -> EntityEvents<PublicIdEvent> {
        EntityEvents::init(
            self.id.clone(),
            [PublicIdEvent::Initialized {
                id: self.id,
                target_id: self.target_id,
                target_type: self.target_type,
            }],
        )
    }
}
