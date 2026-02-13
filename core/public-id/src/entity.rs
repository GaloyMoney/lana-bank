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
        target_type: PublicIdTargetType,
    },
    TargetUpdated {
        target_id: PublicIdTargetId,
        target_type: PublicIdTargetType,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PublicIdEntity {
    pub id: PublicId,
    pub target_id: PublicIdTargetId,
    pub target_type: PublicIdTargetType,
    events: EntityEvents<PublicIdEntityEvent>,
}

impl core::fmt::Display for PublicIdEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicId: {}", self.id)
    }
}

impl PublicIdEntity {
    pub fn update_target(
        &mut self,
        target_id: impl Into<PublicIdTargetId>,
        target_type: impl Into<PublicIdTargetType>,
    ) -> Idempotent<()> {
        let target_id = target_id.into();
        let target_type = target_type.into();
        idempotency_guard!(
            self.events.iter_all().rev(),
            PublicIdEntityEvent::TargetUpdated {
                target_id: existing_id,
                target_type: existing_type,
            } if *existing_id == target_id && existing_type == &target_type
        );
        self.events.push(PublicIdEntityEvent::TargetUpdated {
            target_id,
            target_type: target_type.clone(),
        });
        self.target_id = target_id;
        self.target_type = target_type;
        Idempotent::Executed(())
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
                PublicIdEntityEvent::TargetUpdated {
                    target_id,
                    target_type,
                } => {
                    builder = builder
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
    pub(super) target_type: PublicIdTargetType,
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
