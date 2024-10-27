use sqlx::PgPool;

use es_entity::*;

use crate::{
    data_export::Export,
    primitives::{CreditFacilityId, InterestAccrualId, InterestAccrualIdx},
};

use super::{entity::*, InterestAccrualError};

const BQ_TABLE_NAME: &str = "interest_accrual_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "InterestAccrual",
    err = "InterestAccrualError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", update(persist = false), list_for),
        idx(ty = "InterestAccrualIdx", update(persist = false)),
    ),
    post_persist_hook = "export"
)]
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

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        events: impl Iterator<Item = &PersistedEvent<InterestAccrualEvent>>,
    ) -> Result<(), InterestAccrualError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }
}
