use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "Ref")]
pub enum PublicRefEvent {
    Initialized {
        id: Ref,
        target_id: RefTargetId,
        target_type: RefTargetType,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PublicRef {
    pub id: Ref,
    pub target_id: RefTargetId,
    pub target_type: RefTargetType,
    events: EntityEvents<PublicRefEvent>,
}

impl core::fmt::Display for PublicRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicRef: {}", self.id)
    }
}

impl TryFromEvents<PublicRefEvent> for PublicRef {
    fn try_from_events(events: EntityEvents<PublicRefEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PublicRefBuilder::default();

        for event in events.iter_all() {
            match event {
                PublicRefEvent::Initialized {
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
pub struct NewPublicRef {
    #[builder(setter(into))]
    pub(super) id: Ref,
    #[builder(setter(into))]
    pub(super) target_id: RefTargetId,
    #[builder(setter(into))]
    pub(super) target_type: RefTargetType,
}

impl NewPublicRef {
    pub fn builder() -> NewPublicRefBuilder {
        NewPublicRefBuilder::default()
    }
}

impl IntoEvents<PublicRefEvent> for NewPublicRef {
    fn into_events(self) -> EntityEvents<PublicRefEvent> {
        EntityEvents::init(
            self.id.clone(),
            [PublicRefEvent::Initialized {
                id: self.id,
                target_id: self.target_id,
                target_type: self.target_type,
            }],
        )
    }
}
