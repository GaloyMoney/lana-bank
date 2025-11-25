mod entity;
pub mod error;
mod repo;

use cala_ledger::AccountId as CalaAccountId;
use core_money::UsdCents;
#[cfg(feature = "json-schema")]
pub use entity::LiquidationProcessEvent;
pub(crate) use entity::*;
use error::LiquidationProcessError;
use outbox::OutboxEventMarker;
pub(crate) use repo::LiquidationProcessRepo;

use crate::{CoreCreditEvent, LiquidationProcessId};

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

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, LiquidationProcessError> {
        Ok(self.repo.begin_op().await?)
    }

    pub async fn record_collateral_sent_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        liquidation_process_id: LiquidationProcessId,
    ) -> Result<(), LiquidationProcessError> {
        let mut liquidation = self.repo.find_by_id(liquidation_process_id).await?;

        liquidation.record_collateral_sent_out(todo!(), todo!());

        self.repo.update_in_op(db, &mut liquidation).await?;

        Ok(())
    }

    // expost in GQL
    pub async fn record_payment_from_liquidation(
        &self,
        liquidation_process_id: LiquidationProcessId,
        amount: UsdCents,
    ) -> Result<(), LiquidationProcessError> {
        let mut liquidation = self.repo.find_by_id(liquidation_process_id).await?;
        let mut db = self.repo.begin().await?;

        // post transaction in op

        if liquidation
            .record_repayment_from_liquidation(amount, todo!())?
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
        todo!()
    }
}
