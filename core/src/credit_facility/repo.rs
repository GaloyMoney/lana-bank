use sqlx::{PgPool, Postgres, Transaction};

use crate::{data_export::Export, entity::*, primitives::*};

use super::{entity::*, error::CreditFacilityError, CreditFacilityByCreatedAtCursor};

const BQ_TABLE_NAME: &str = "credit_facility_events";

#[derive(Clone)]
pub struct CreditFacilityRepo {
    pool: PgPool,
    export: Export,
}

impl CreditFacilityRepo {
    pub(super) fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_credit_facility: NewCreditFacility,
    ) -> Result<CreditFacility, CreditFacilityError> {
        sqlx::query!(
            r#"INSERT INTO credit_facilities (id, customer_id)
            VALUES ($1, $2)"#,
            new_credit_facility.id as CreditFacilityId,
            new_credit_facility.customer_id as CustomerId,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_credit_facility.initial_events();
        let n_events = events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &events)
            .await?;
        Ok(CreditFacility::try_from(events)?)
    }

    pub(super) async fn persist_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        credit_facility: &mut CreditFacility,
    ) -> Result<(), CreditFacilityError> {
        let n_events = credit_facility.events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &credit_facility.events)
            .await?;
        Ok(())
    }

    pub async fn find_by_id(
        &self,
        id: CreditFacilityId,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT c.id, e.sequence, e.event,
                      c.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM credit_facilities c
            JOIN credit_facility_events e ON c.id = e.id
            WHERE c.id = $1
            ORDER BY e.sequence"#,
            id as CreditFacilityId,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<CreditFacility>(rows)?;
        Ok(res)
    }

    pub async fn find_for_customer(
        &self,
        customer_id: CustomerId,
    ) -> Result<Vec<CreditFacility>, CreditFacilityError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT l.id, e.sequence, e.event,
                      l.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM credit_facilities l
            JOIN credit_facility_events e ON l.id = e.id
            WHERE l.customer_id = $1
            ORDER BY l.id, e.sequence"#,
            customer_id as CustomerId,
        )
        .fetch_all(&self.pool)
        .await?;

        let n = rows.len();
        let res = EntityEvents::load_n::<CreditFacility>(rows, n)?;
        Ok(res.0)
    }

    pub async fn list(
        &self,
        query: crate::query::PaginatedQueryArgs<CreditFacilityByCreatedAtCursor>,
    ) -> Result<
        crate::query::PaginatedQueryRet<CreditFacility, CreditFacilityByCreatedAtCursor>,
        CreditFacilityError,
    > {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"
            WITH credit_facilities AS (
              SELECT id, customer_id, created_at
              FROM credit_facilities
              WHERE ((created_at, id) < ($2, $1)) OR ($1 IS NULL AND $2 IS NULL)
              ORDER BY created_at DESC, id DESC
              LIMIT $3
            )
            SELECT l.id, e.sequence, e.event,
              l.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM credit_facilities l
            JOIN credit_facility_events e ON l.id = e.id
            ORDER BY l.created_at DESC, l.id DESC, e.sequence;
            "#,
            query.after.as_ref().map(|c| c.id) as Option<CreditFacilityId>,
            query.after.map(|l| l.created_at),
            query.first as i64 + 1
        )
        .fetch_all(&self.pool)
        .await?;
        let (entities, has_next_page) = EntityEvents::load_n::<CreditFacility>(rows, query.first)?;
        let mut end_cursor = None;
        if let Some(last) = entities.last() {
            end_cursor = Some(CreditFacilityByCreatedAtCursor {
                id: last.id,
                created_at: last.created_at(),
            });
        }
        Ok(crate::query::PaginatedQueryRet {
            entities,
            has_next_page,
            end_cursor,
        })
    }
}
