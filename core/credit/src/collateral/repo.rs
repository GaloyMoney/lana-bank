use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    event::CoreCreditEvent,
    primitives::{CollateralId, CreditFacilityId, CustodyWalletId, LiquidationId},
    publisher::CreditFacilityPublisher,
};

use super::{
    entity::*,
    error::*,
    liquidation::{Liquidation, LiquidationError, LiquidationEvent},
};

#[derive(EsRepo)]
#[es_repo(
    entity = "Collateral",
    err = "CollateralError",
    columns(custody_wallet_id(ty = "Option<CustodyWalletId>", update(persist = false))),
    tbl_prefix = "core",
    post_persist_hook = "publish_in_op"
)]
pub struct CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pool: PgPool,
    publisher: CreditFacilityPublisher<E>,
    clock: ClockHandle,

    #[es_repo(nested)]
    liquidations: LiquidationRepo,
}

impl<E> CollateralRepo<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &PgPool, publisher: &CreditFacilityPublisher<E>, clock: ClockHandle) -> Self {
        let liquidations = LiquidationRepo::new(pool, clock.clone());
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
            clock,
            liquidations,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "collateral.publish_in_op", skip_all)]
    async fn publish_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Collateral,
        new_events: es_entity::LastPersisted<'_, CollateralEvent>,
    ) -> Result<(), CollateralError> {
        self.publisher
            .publish_collateral_in_op(op, entity, new_events)
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

#[derive(EsRepo)]
#[es_repo(
    entity = "Liquidation",
    err = "LiquidationError",
    columns(
        collateral_id(ty = "CollateralId", list_for, parent, update(persist = false)),
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        completed(
            ty = "bool",
            create(persist = false),
            update(accessor = "is_completed()")
        )
    ),
    tbl_prefix = "core"
)]
pub(super) struct LiquidationRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl Clone for LiquidationRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl LiquidationRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}
