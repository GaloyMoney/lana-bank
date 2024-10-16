use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    data_export::Export,
    entity::{EntityEvents, GenericEvent},
    primitives::{CreditFacilityId, InterestAccrualId, InterestAccrualIdx},
};

use super::{entity::*, InterestAccrualError};

const BQ_TABLE_NAME: &str = "interest_accrual_events";

#[derive(Clone)]
pub(in crate::credit_facility) struct InterestAccrualRepo {
    pool: PgPool,
    export: Export,
}

impl InterestAccrualRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_interest_accrual: NewInterestAccrual,
    ) -> Result<InterestAccrual, InterestAccrualError> {
        sqlx::query!(
            r#"INSERT INTO interest_accruals (id, credit_facility_id, idx)
            VALUES ($1, $2, $3)"#,
            new_interest_accrual.id as InterestAccrualId,
            new_interest_accrual.facility_id as CreditFacilityId,
            new_interest_accrual.idx as InterestAccrualIdx,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_interest_accrual.initial_events();
        let n_events = events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &events)
            .await?;
        Ok(InterestAccrual::try_from(events)?)
    }

    pub async fn persist_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        interest_accrual: &mut InterestAccrual,
    ) -> Result<(), InterestAccrualError> {
        let n_events = interest_accrual.events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &interest_accrual.events)
            .await?;
        Ok(())
    }

    pub async fn find_by_idx_for_credit_facility(
        &self,
        facility_id: CreditFacilityId,
        idx: InterestAccrualIdx,
    ) -> Result<InterestAccrual, InterestAccrualError> {
        let rows = sqlx::query_as!(
            GenericEvent,
            r#"SELECT i.id, e.sequence, e.event,
                      i.created_at AS entity_created_at, e.recorded_at AS event_recorded_at
            FROM interest_accruals i
            JOIN interest_accrual_events e ON i.id = e.id
            WHERE i.credit_facility_id = $1 AND i.idx = $2
            ORDER BY e.sequence"#,
            facility_id as CreditFacilityId,
            idx as InterestAccrualIdx,
        )
        .fetch_all(&self.pool)
        .await?;

        let res = EntityEvents::load_first::<InterestAccrual>(rows)?;
        Ok(res)
    }
}
