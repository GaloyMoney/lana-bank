use sqlx::PgPool;

use es_entity::*;

use crate::{data_export::Export, primitives::*};

use super::{entity::*, error::*};

const BQ_TABLE_NAME: &str = "user_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "User",
    err = "UserError",
    columns(email = "String"),
    post_persist_hook = "export"
)]
pub struct UserRepo {
    pool: PgPool,
    export: Export,
}

impl UserRepo {
    pub(super) fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        events: impl Iterator<Item = &PersistedEvent<UserEvent>>,
    ) -> Result<(), UserError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<User>, UserError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM users a
            JOIN user_events e
            ON a.id = e.id
            ORDER BY a.email, a.id, e.sequence"#,
        )
        .fetch_all(&self.pool)
        .await?;
        let n = rows.len();
        let res = EntityEvents::load_n::<User>(rows, n)?;
        Ok(res.0)
    }
}
