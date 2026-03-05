use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "NoteId")]
pub enum NoteEvent {
    Initialized {
        id: NoteId,
        target_type: NoteTargetType,
        target_id: String,
        content: String,
    },
    Updated {
        content: String,
    },
    Deleted,
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct Note {
    pub id: NoteId,
    pub target_type: NoteTargetType,
    pub target_id: String,
    pub content: String,
    events: EntityEvents<NoteEvent>,
}

impl core::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Note: {}", self.id)
    }
}

impl Note {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn update_content(&mut self, content: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            NoteEvent::Updated { content: existing_content } if existing_content == &content
        );
        self.content = content.clone();
        self.events.push(NoteEvent::Updated { content });
        Idempotent::Executed(())
    }

    pub fn delete(&mut self) -> Idempotent<()> {
        idempotency_guard!(self.events.iter_all().rev(), NoteEvent::Deleted);
        self.events.push(NoteEvent::Deleted);
        Idempotent::Executed(())
    }
}

impl TryFromEvents<NoteEvent> for Note {
    fn try_from_events(events: EntityEvents<NoteEvent>) -> Result<Self, EntityHydrationError> {
        let mut builder = NoteBuilder::default();

        for event in events.iter_all() {
            match event {
                NoteEvent::Initialized {
                    id,
                    target_type,
                    target_id,
                    content,
                } => {
                    builder = builder
                        .id(*id)
                        .target_type(target_type.clone())
                        .target_id(target_id.clone())
                        .content(content.clone());
                }
                NoteEvent::Updated { content } => {
                    builder = builder.content(content.clone());
                }
                NoteEvent::Deleted => {}
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct NewNote {
    #[builder(setter(into))]
    pub(super) id: NoteId,
    pub(super) target_type: NoteTargetType,
    pub(super) target_id: String,
    pub(super) content: String,
}

impl NewNote {
    pub fn builder() -> NewNoteBuilder {
        NewNoteBuilder::default()
    }
}

impl IntoEvents<NoteEvent> for NewNote {
    fn into_events(self) -> EntityEvents<NoteEvent> {
        EntityEvents::init(
            self.id,
            [NoteEvent::Initialized {
                id: self.id,
                target_type: self.target_type,
                target_id: self.target_id,
                content: self.content,
            }],
        )
    }
}
