use sqlx::{PgPool, Postgres, Transaction};
use tracing::instrument;

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
            r#"INSERT INTO credit_facilities (id)
            VALUES ($1)"#,
            new_credit_facility.id as CreditFacilityId,
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
}
