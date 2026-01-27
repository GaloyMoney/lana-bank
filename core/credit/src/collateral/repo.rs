use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    event::CoreCreditEvent,
    liquidation::LiquidationRepo,
    primitives::{CollateralId, CreditFacilityId, CustodyWalletId},
    publisher::CreditFacilityPublisher,
};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Collateral",
    err = "CollateralError",
    columns(custody_wallet_id(ty = "Option<CustodyWalletId>", update(persist = false))),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,

    #[es_repo(nested)]
    liquidations: LiquidationRepo<E>,
}

impl<E> CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>, clock: ClockHandle) -> Self {
        let liquidations = LiquidationRepo::new(pool, publisher, clock.clone());
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
            liquidations,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "collateral.publish", skip_all)]
    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Collateral,
        new_events: es_entity::LastPersisted<'_, CollateralEvent>,
    ) -> Result<(), CollateralError> {
        self.publisher
            .publish_collateral(op, entity, new_events)
            .await
    }

    #[record_error_severity]
    #[tracing::instrument(name = "collateral.find_by_credit_facility_id", skip(self))]
    pub async fn find_by_credit_facility_id(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Collateral, CollateralError> {
        es_query!(
            tbl_prefix = "core",
            r#"
            SELECT c.id FROM core_collaterals c
            INNER JOIN core_collateral_events ce ON c.id = ce.id
            WHERE ce.event_type = 'initialized'
              AND (ce.event ->> 'credit_facility_id')::UUID = $1"#,
            credit_facility_id as CreditFacilityId
        )
        .fetch_one(&mut self.pool().begin().await?)
        .await
    }
}

impl<E> Clone for CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
            clock: self.clock.clone(),
            liquidations: self.liquidations.clone(),
        }
    }
}
