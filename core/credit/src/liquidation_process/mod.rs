mod entity;
pub mod error;
mod repo;

use cala_ledger::{AccountId as CalaAccountId, TransactionId as CalaTransactionId};
use core_money::{Satoshis, UsdCents};
use core_price::PriceOfOneBTC;
#[cfg(feature = "json-schema")]
pub use entity::LiquidationProcessEvent;
pub(crate) use entity::*;
use error::LiquidationProcessError;
use es_entity::DbOp;
use outbox::OutboxEventMarker;
pub(crate) use repo::LiquidationProcessRepo;

use crate::{CoreCreditEvent, CreditFacilityId, LiquidationProcessId};

pub struct Liquidations<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: LiquidationProcessRepo<E>,
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
            repo: LiquidationProcessRepo::new(pool, publisher),
        }
    }

    pub async fn create_if_not_exist_in_op(
        &self,
        db: &mut DbOp<'_>,
        liquidation_process_id: LiquidationProcessId,
        credit_facility_id: CreditFacilityId,
        receivable_account_id: CalaAccountId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    ) -> Result<Option<LiquidationProcess>, LiquidationProcessError> {
        match self
            .repo
            .maybe_find_by_credit_facility_id_in_op(&mut *db, credit_facility_id)
            .await?
        {
            None => {
                let new_liquidation = NewLiquidationProcess {
                    id: liquidation_process_id,
                    credit_facility_id,
                    receivable_account_id,
                    trigger_price,
                    initially_expected_to_receive,
                    initially_estimated_to_liquidate,
                };
                let liquidation = self.repo.create_in_op(db, new_liquidation).await?;
                Ok(Some(liquidation))
            }
            _ => Ok(None),
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, LiquidationProcessError> {
        Ok(self.repo.begin_op().await?)
    }

    #[allow(dead_code)]
    pub async fn record_collateral_sent_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        liquidation_process_id: LiquidationProcessId,
        amount: Satoshis,
    ) -> Result<(), LiquidationProcessError> {
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
        liquidation_process_id: LiquidationProcessId,
        amount: UsdCents,
    ) -> Result<(), LiquidationProcessError> {
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
        liquidation_process_id: LiquidationProcessId,
    ) -> Result<(), LiquidationProcessError> {
        let mut liquidation = self.repo.find_by_id(liquidation_process_id).await?;

        if liquidation.complete().did_execute() {
            self.repo.update_in_op(db, &mut liquidation).await?;
        }

        Ok(())
    }
}
