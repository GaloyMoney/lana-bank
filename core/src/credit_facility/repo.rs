use sqlx::{PgPool, Postgres, Transaction};

use crate::{data_export::Export, entity::*, primitives::*};

use super::{entity::*, error::CreditFacilityError};

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
}
