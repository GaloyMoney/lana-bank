use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Note",
    columns(target_type(ty = "NoteTargetType"), target_id(ty = "String"),),
    tbl_prefix = "core",
    delete = "soft_without_queries"
)]
pub struct NoteRepo {
    pool: PgPool,
}

impl Clone for NoteRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl NoteRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_by_target(
        &self,
        target_type: &NoteTargetType,
        target_id: &str,
    ) -> Result<Vec<Note>, NoteError> {
        let target_type_str = target_type.as_str();
        let ids: Vec<NoteId> = sqlx::query_scalar!(
            r#"SELECT id AS "id: NoteId"
            FROM core_notes
            WHERE target_type = $1 AND target_id = $2 AND deleted = FALSE
            ORDER BY created_at"#,
            target_type_str,
            target_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut notes_map = self.find_all::<Note>(&ids).await?;
        let notes = ids
            .into_iter()
            .filter_map(|id| notes_map.remove(&id))
            .collect();
        Ok(notes)
    }
}
