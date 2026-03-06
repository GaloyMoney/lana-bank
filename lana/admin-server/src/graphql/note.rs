use async_graphql::*;

use crate::primitives::*;
use lana_app::note::{Note as DomainNote, NoteTargetKind};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Note {
    id: ID,
    note_id: UUID,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainNote>,
}

impl From<DomainNote> for Note {
    fn from(note: DomainNote) -> Self {
        Note {
            id: note.id.to_global_id(),
            note_id: UUID::from(note.id),
            created_at: note.created_at().into(),
            entity: Arc::new(note),
        }
    }
}

#[ComplexObject]
impl Note {
    async fn target_type(&self) -> async_graphql::Result<NoteTargetKind> {
        NoteTargetKind::try_from(&self.entity.target_type)
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn target_id(&self) -> &str {
        &self.entity.target_id
    }

    async fn content(&self) -> &str {
        &self.entity.content
    }
}

#[derive(InputObject)]
pub struct NoteCreateInput {
    pub target_type: NoteTargetKind,
    pub target_id: UUID,
    pub content: String,
}
crate::mutation_payload! { NoteCreatePayload, note: Note }

#[derive(InputObject)]
pub struct NoteUpdateInput {
    pub note_id: UUID,
    pub content: String,
}
crate::mutation_payload! { NoteUpdatePayload, note: Note }

#[derive(InputObject)]
pub struct NoteDeleteInput {
    pub note_id: UUID,
}

#[derive(SimpleObject)]
pub struct NoteDeletePayload {
    pub deleted_note_id: UUID,
}
