use sqlx::PgPool;

use es_entity::*;

use crate::{data_export::Export, primitives::*};

use super::{entity::*, error::*};

const BQ_TABLE_NAME: &str = "terms_template_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "TermsTemplate",
    err = "TermsTemplateError",
    columns(name = "String"),
    post_persist_hook = "export"
)]
pub struct TermsTemplateRepo {
    pool: PgPool,
    export: Export,
}

impl TermsTemplateRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        events: impl Iterator<Item = &PersistedEvent<TermsTemplateEvent>>,
    ) -> Result<(), TermsTemplateError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }
}
