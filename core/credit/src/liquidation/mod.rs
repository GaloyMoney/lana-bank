mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use cala_ledger::TransactionId as CalaTransactionId;
use core_money::{Satoshis, UsdCents};
use es_entity::DbOp;
use outbox::OutboxEventMarker;

use crate::{CoreCreditEvent, CreditFacilityId, LiquidationId, PaymentId};
pub use entity::LiquidationEvent;
pub(crate) use entity::*;
use error::LiquidationError;
pub(crate) use repo::LiquidationRepo;

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

    #[instrument(
        name = "credit.liquidation.create_if_not_exist_for_facility_in_op",
        skip(self, db, new_liquidation),
        fields(existing_liquidation_found),
        err
    )]
    pub async fn create_if_not_exist_for_facility_in_op(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        new_liquidation: NewLiquidation,
    ) -> Result<Option<Liquidation>, LiquidationError> {
        let existing_liquidation = self
            .repo
            .maybe_find_active_liquidation_for_credit_facility_id_in_op(
                &mut *db,
                credit_facility_id,
            )
            .await?;

        tracing::Span::current()
            .record("existing_liquidation_found", existing_liquidation.is_some());

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

    #[instrument(
        name = "credit.liquidation.record_collateral_sent_in_op",
        skip(self, db),
        err
    )]
    #[allow(dead_code)]
    pub async fn record_collateral_sent_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        liquidation_id: LiquidationId,
        amount: Satoshis,
    ) -> Result<(), LiquidationError> {
        let mut liquidation = self.repo.find_by_id(liquidation_id).await?;

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

    #[instrument(
        name = "credit.liquidation.record_payment_from_liquidation",
        skip(self),
        err
    )]
    #[allow(dead_code)]
    pub async fn record_payment_from_liquidation(
        &self,
        liquidation_id: LiquidationId,
        amount: UsdCents,
    ) -> Result<(), LiquidationError> {
        let mut liquidation = self.repo.find_by_id(liquidation_id).await?;
        let mut db = self.repo.begin_op().await?;

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

    #[instrument(name = "credit.liquidation.complete_in_op", skip(self, db), err)]
    pub async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        liquidation_id: LiquidationId,
        payment_id: PaymentId,
    ) -> Result<(), LiquidationError> {
        let mut liquidation = self.repo.find_by_id(liquidation_id).await?;

        if liquidation.complete(payment_id).did_execute() {
            self.repo.update_in_op(db, &mut liquidation).await?;
        }

        Ok(())
    }

    pub async fn list_active(&self) -> Result<Vec<Liquidation>, LiquidationError> {
        self.repo.list_active().await
    }
}
