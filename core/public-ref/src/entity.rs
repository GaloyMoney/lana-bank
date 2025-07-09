use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PublicRefId")]
pub enum PublicRefEvent {
    Initialized {
        id: PublicRefId,
        reference: Ref,
        target_type: RefTargetType,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PublicRef {
    pub id: PublicRefId,
    pub reference: Ref,
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
                    reference,
                    target_type: target,
                } => {
                    builder = builder
                        .id(*id)
                        .reference(reference.clone())
                        .target_type(target.clone());
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
    pub(super) id: PublicRefId,
    #[builder(setter(into))]
    pub(super) reference: Ref,
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
            self.id,
            [PublicRefEvent::Initialized {
                id: self.id,
                reference: self.reference,
                target_type: self.target_type,
            }],
        )
    }
}
