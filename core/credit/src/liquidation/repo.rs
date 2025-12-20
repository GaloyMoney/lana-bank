use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{event::CoreCreditEvent, primitives::*, publisher::CreditFacilityPublisher};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Liquidation",
    err = "LiquidationError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        completed(
            ty = "bool",
            create(persist = false),
            update(accessor = "is_completed()")
        )
    ),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct LiquidationRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
}

impl<E> Clone for LiquidationRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}

impl<E> LiquidationRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "liquidation.publish", skip_all)]
    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Liquidation,
        new_events: es_entity::LastPersisted<'_, LiquidationEvent>,
    ) -> Result<(), LiquidationError> {
        self.publisher
            .publish_liquidation(op, entity, new_events)
            .await
    }

    #[tracing::instrument(
        name = "liquidation.maybe_find_active_liquidation_by_credit_facility_id_in_op",
        skip(self, db),
        err
    )]
    pub async fn maybe_find_active_liquidation_for_credit_facility_id_in_op(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Option<Liquidation>, LiquidationError> {
        let res = es_entity::es_query!(
            entity = Liquidation,
            r#"
            SELECT *
            FROM core_liquidations
            WHERE credit_facility_id = $1
              AND completed = false
            "#,
            credit_facility_id as CreditFacilityId
        )
        .fetch_optional(db)
        .await?;

        Ok(res)
    }
}
