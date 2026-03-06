use async_graphql::*;

use crate::primitives::*;
use lana_app::note::Note as DomainNote;

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
    async fn content(&self) -> &str {
        &self.entity.content
    }
}

#[derive(InputObject)]
pub struct CustomerNoteCreateInput {
    pub customer_id: UUID,
    pub content: String,
}
crate::mutation_payload! { CustomerNoteCreatePayload, note: Note }

#[derive(InputObject)]
pub struct CustomerNoteUpdateInput {
    pub note_id: UUID,
    pub content: String,
}
crate::mutation_payload! { CustomerNoteUpdatePayload, note: Note }

#[derive(InputObject)]
pub struct CustomerNoteDeleteInput {
    pub note_id: UUID,
}

#[derive(SimpleObject)]
pub struct CustomerNoteDeletePayload {
    pub deleted_note_id: UUID,
}
