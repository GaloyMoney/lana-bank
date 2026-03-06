#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use tracing::instrument;
use tracing_macros::record_error_severity;

pub use entity::{NewNote, Note};
pub use error::*;
pub use primitives::*;
pub use repo::{NoteRepo, note_cursor};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::NoteEvent;
}

pub struct Notes {
    repo: NoteRepo,
}

impl Clone for Notes {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

impl Notes {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = NoteRepo::new(pool);
        Self { repo }
    }

    #[record_error_severity]
    #[instrument(name = "core_note.create", skip(self))]
    pub async fn create(
        &self,
        target_type: NoteTargetType,
        target_id: impl Into<NoteTargetId> + std::fmt::Debug,
        content: String,
    ) -> Result<Note, NoteError> {
        let new_note = NewNote::builder()
            .id(NoteId::new())
            .target_type(target_type)
            .target_id(target_id)
            .content(content)
            .build()
            .expect("Could not build note");

        let note = self.repo.create(new_note).await?;
        Ok(note)
    }

    #[record_error_severity]
    #[instrument(name = "core_note.update", skip(self))]
    pub async fn update(&self, id: NoteId, content: String) -> Result<Note, NoteError> {
        let mut note = self.repo.find_by_id(id).await?;
        if note.update_content(content).did_execute() {
            self.repo.update(&mut note).await?;
        }
        Ok(note)
    }

    #[record_error_severity]
    #[instrument(name = "core_note.delete", skip(self))]
    pub async fn delete(&self, id: NoteId) -> Result<(), NoteError> {
        let mut note = self.repo.find_by_id(id).await?;
        if note.delete().did_execute() {
            self.repo.delete(note).await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "core_note.find_by_id", skip(self))]
    pub async fn find_by_id(&self, id: NoteId) -> Result<Option<Note>, NoteError> {
        Ok(self.repo.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "core_note.list_for_target", skip(self))]
    pub async fn list_for_target(
        &self,
        target_id: impl Into<NoteTargetId> + std::fmt::Debug,
        query: es_entity::PaginatedQueryArgs<note_cursor::NotesByCreatedAtCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Note, note_cursor::NotesByCreatedAtCursor>, NoteError>
    {
        Ok(self
            .repo
            .list_for_target_id_by_created_at(
                target_id.into(),
                query,
                es_entity::ListDirection::Ascending,
            )
            .await?)
    }

    pub async fn find_all(
        &self,
        ids: &[NoteId],
    ) -> Result<std::collections::HashMap<NoteId, Note>, NoteError> {
        Ok(self.repo.find_all(ids).await?)
    }
}
