use rust_decimal::Decimal;
use sqlx::PgPool;

use es_entity::*;

use crate::{
    data_export::Export,
    primitives::{CustomerId, LoanId},
};

use super::{entity::*, error::LoanError};

const BQ_TABLE_NAME: &str = "loan_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Loan",
    err = "LoanError",
    columns(
        customer_id(ty = "CustomerId", list_for),
        collateralization_ratio(
            ty = "Option<Decimal>",
            create(persist = false),
            update(accessor = "collateralization_ratio()")
        ),
    ),
    post_persist_hook = "export"
)]
pub struct LoanRepo {
    pool: PgPool,
    export: Export,
}

impl LoanRepo {
    pub(super) fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        events: impl Iterator<Item = &PersistedEvent<LoanEvent>>,
    ) -> Result<(), LoanError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }
}
