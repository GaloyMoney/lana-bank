mod entity;
pub mod error;
mod repo;

use cala_ledger::{AccountId as CalaAccountId, TransactionId as CalaTransactionId};
use core_money::{Satoshis, UsdCents};
use core_price::PriceOfOneBTC;
#[cfg(feature = "json-schema")]
pub use entity::LiquidationEvent;
pub(crate) use entity::*;
use error::LiquidationError;
use es_entity::DbOp;
use outbox::OutboxEventMarker;
pub(crate) use repo::LiquidationRepo;

use crate::{CoreCreditEvent, CreditFacilityId, LiquidationId};

pub struct Liquidations<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: LiquidationRepo<E>,
}

impl<E> Clone for Liquidations<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

impl<E> Liquidations<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &sqlx::PgPool, publisher: &crate::CreditFacilityPublisher<E>) -> Self {
        Self {
            repo: LiquidationRepo::new(pool, publisher),
        }
    }

    pub async fn create_if_not_exist_for_facility_in_op(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        new_liqiudation: NewLiquidation,
    ) -> Result<Option<Liquidation>, LiquidationError> {
        let existing_liquidation = self
            .repo
            .maybe_find_by_credit_facility_id_in_op(&mut *db, credit_facility_id)
            .await?;

        if existing_liquidation.is_none() {
            let liquidation = self.repo.create_in_op(db, new_liquidation).await?;
            Ok(Some(liquidation))
        } else {
            Ok(None)
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, LiquidationError> {
        Ok(self.repo.begin_op().await?)
    }

    #[allow(dead_code)]
    pub async fn record_collateral_sent_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        liquidation_process_id: LiquidationId,
        amount: Satoshis,
    ) -> Result<(), LiquidationError> {
        let mut liquidation = self.repo.find_by_id(liquidation_process_id).await?;

        let tx_id = CalaTransactionId::new();

        if liquidation
            .record_collateral_sent_out(amount, tx_id)?
            .did_execute()
        {
            self.repo.update_in_op(db, &mut liquidation).await?;
            // TODO: ledger send collateral for liquidation
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn record_payment_from_liquidation(
        &self,
        liquidation_process_id: LiquidationId,
        amount: UsdCents,
    ) -> Result<(), LiquidationError> {
        let mut liquidation = self.repo.find_by_id(liquidation_process_id).await?;
        let mut db = self.repo.begin().await?;

        // TODO: post transaction in op
        let tx_id = CalaTransactionId::new();

        if liquidation
            .record_repayment_from_liquidation(amount, tx_id)?
            .did_execute()
        {
            self.repo.update_in_op(&mut db, &mut liquidation).await?;
        }

        Ok(())
    }

    pub async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        liquidation_process_id: LiquidationId,
    ) -> Result<(), LiquidationError> {
        let mut liquidation = self.repo.find_by_id(liquidation_process_id).await?;

        if liquidation.complete().did_execute() {
            self.repo.update_in_op(db, &mut liquidation).await?;
        }

        Ok(())
    }
}
