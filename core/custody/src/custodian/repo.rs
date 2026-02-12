use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use tracing_macros::record_error_severity;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Custodian",
    err = "CustodianError",
    columns(name(ty = "String", list_by), provider(ty = "String", find_by)),
    tbl_prefix = "core"
)]
pub(crate) struct CustodianRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl CustodianRepo {
    pub(crate) fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }

    pub(crate) async fn list_all(&self) -> Result<Vec<Custodian>, CustodianError> {
        let mut custodians = Vec::new();
        let mut next = Some(PaginatedQueryArgs::default());

        while let Some(query) = next.take() {
            let mut ret = self.list_by_id(query, Default::default()).await?;

            custodians.append(&mut ret.entities);
            next = ret.into_next_query();
        }

        Ok(custodians)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "custodian.update_config_in_op", skip_all)]
    pub async fn update_config_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        custodian: &mut Custodian,
    ) -> Result<(), CustodianError> {
        sqlx::query!(
            r#"
            UPDATE core_custodian_events
            SET event = jsonb_set(event, '{encrypted_custodian_config}', 'null'::jsonb, false)
            WHERE id = $1 
              AND event_type = 'config_updated'
              AND event->'encrypted_custodian_config' IS NOT NULL;
            "#,
            custodian.id as CustodianId,
        )
        .execute(op.as_executor())
        .await?;

        self.update_in_op(op, custodian).await?;

        Ok(())
    }
}
