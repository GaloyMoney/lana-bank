use sqlx::PgPool;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(Clone)]
pub struct ReportRepo {
    pool: PgPool,
}

impl ReportRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub(super) async fn create_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        new_report: NewReport,
    ) -> Result<Report, ReportError> {
        sqlx::query!(
            r#"INSERT INTO reports (id)
            VALUES ($1)"#,
            new_report.id as ReportId,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_report.initial_events();
        events.persist(db).await?;
        Ok(Report::try_from(events)?)
    }
}
