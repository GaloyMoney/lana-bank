use sqlx::{PgPool, Postgres, Transaction};

use crate::{data_export::Export, entity::*, primitives::LoanTermsId};

use super::{
    error::TermsTemplateError,
    template::{NewTermsTemplate, TermsTemplate},
};

const BQ_TABLE_NAME: &str = "terms_template_events";

#[derive(Clone)]
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

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_template: NewTermsTemplate,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        sqlx::query!(
            r#"INSERT INTO terms_templates (id, name)
            VALUES ($1, $2)
            "#,
            new_template.id as LoanTermsId,
            new_template.name,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_template.initial_events();
        let n_events = events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &events)
            .await?;
        Ok(TermsTemplate::try_from(events)?)
    }

    pub async fn find_by_id(&self, id: LoanTermsId) -> Result<TermsTemplate, TermsTemplateError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM terms_templates a
            JOIN terms_template_events e
            ON a.id = e.id
            WHERE a.id = $1"#,
            id as LoanTermsId
        )
        .fetch_all(&self.pool)
        .await?;
        match EntityEvents::load_first(rows) {
            Ok(template) => Ok(template),
            Err(EntityError::NoEntityEventsPresent) => {
                Err(TermsTemplateError::CouldNotFindById(id))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn list(&self) -> Result<Vec<TermsTemplate>, TermsTemplateError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM terms_templates a
            JOIN terms_template_events e
            ON a.id = e.id
            ORDER BY a.name, a.id, e.sequence"#,
        )
        .fetch_all(&self.pool)
        .await?;
        let n = rows.len();
        let res = EntityEvents::load_n::<TermsTemplate>(rows, n)?;
        Ok(res.0)
    }

    pub async fn _persist_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        template: &mut TermsTemplate,
    ) -> Result<(), TermsTemplateError> {
        let n_events = template.events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &template.events)
            .await?;
        Ok(())
    }
}
