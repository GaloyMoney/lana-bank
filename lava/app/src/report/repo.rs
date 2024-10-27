use sqlx::{PgPool, Transaction};

use crate::{entity::*, primitives::ReportId};

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
        db: &mut Transaction<'_, sqlx::Postgres>,
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

    pub async fn find_by_id(&self, id: ReportId) -> Result<Report, ReportError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM reports a
            JOIN report_events e
            ON a.id = e.id
            WHERE a.id = $1"#,
            id as ReportId
        )
        .fetch_all(&self.pool)
        .await?;
        match EntityEvents::load_first(rows) {
            Ok(user) => Ok(user),
            Err(EntityError::NoEntityEventsPresent) => Err(ReportError::CouldNotFindById(id)),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn list(&self) -> Result<Vec<Report>, ReportError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT a.id, e.sequence, e.event,
                a.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM reports a
            JOIN report_events e
            ON a.id = e.id
            ORDER BY a.created_at DESC, a.id, e.sequence"#,
        )
        .fetch_all(&self.pool)
        .await?;
        let n = rows.len();
        let res = EntityEvents::load_n::<Report>(rows, n)?;
        Ok(res.0)
    }

    pub async fn update_in_tx(
        &self,
        db: &mut Transaction<'_, sqlx::Postgres>,
        report: &mut Report,
    ) -> Result<(), ReportError> {
        report.events.persist(db).await?;
        Ok(())
    }
}
