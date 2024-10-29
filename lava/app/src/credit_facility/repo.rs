use rust_decimal::Decimal;
use sqlx::PgPool;

use es_entity::*;

use crate::{data_export::Export, primitives::*};

use super::{entity::*, error::CreditFacilityError};

const BQ_TABLE_NAME: &str = "credit_facility_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "CreditFacility",
    err = "CreditFacilityError",
    columns(
        customer_id(ty = "CustomerId", list_for),
        approval_process_id(ty = "ApprovalProcessId", update(persist = "false")),
        collateralization_ratio(
            ty = "Option<Decimal>",
            create(persist = false),
            update(accessor = "collateralization_ratio()")
        ),
    ),
    post_persist_hook = "export"
)]
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

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        events: impl Iterator<Item = &PersistedEvent<CreditFacilityEvent>>,
    ) -> Result<(), CreditFacilityError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }
}
