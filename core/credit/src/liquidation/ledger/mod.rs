mod error;
mod templates;

use core_money::{Satoshis, UsdCents};
use tracing::instrument;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, Currency, JournalId, TransactionId as CalaTransactionId,
};
use tracing_macros::record_error_severity;

pub use error::LiquidationLedgerError;

use crate::LiquidationCompletedData;

#[derive(Clone)]
pub struct LiquidationLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl LiquidationLedger {
    #[record_error_severity]
    #[instrument(name = "core_credit.liquidation.ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
    ) -> Result<Self, LiquidationLedgerError> {
        templates::SendCollateralToLiquidation::init(cala).await?;
        templates::ReceivePaymentFromLiquidation::init(cala).await?;
        templates::CompleteLiquidation::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
        })
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.liquidation.ledger.record_collateral_sent_in_op",
        skip(self, db)
    )]
    pub async fn record_collateral_sent_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        tx_id: CalaTransactionId,
        amount: Satoshis,
        collateral_account_id: CalaAccountId,
        collateral_in_liquidation_account_id: CalaAccountId,
    ) -> Result<(), LiquidationLedgerError> {
        self.cala
            .post_transaction_in_op(
                db,
                tx_id,
                templates::SEND_COLLATERAL_TO_LIQUIDATION,
                templates::SendCollateralToLiquidationParams {
                    amount,
                    journal_id: self.journal_id,
                    collateral_account_id,
                    collateral_in_liquidation_account_id,
                    effective: crate::time::now().date_naive(),
                },
            )
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.liquidation.ledger.record_payment_from_liquidation_in_op",
        skip(self, db)
    )]
    pub async fn record_payment_from_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        tx_id: CalaTransactionId,
        amount: UsdCents,
        omnibus_account_id: CalaAccountId,
        receivable_account_id: CalaAccountId,
    ) -> Result<(), LiquidationLedgerError> {
        self.cala
            .post_transaction_in_op(
                db,
                tx_id,
                templates::RECEIVE_PAYMENT_FROM_LIQUIDATION,
                templates::ReceivePaymentFromLiquidationParams {
                    amount,
                    journal_id: self.journal_id,
                    omnibus_account_id,
                    receivable_account_id,
                    effective: crate::time::now().date_naive(),
                    currency: Currency::USD,
                },
            )
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.liquidation.ledger.complete_liquidation_in_op",
        skip(self, db)
    )]
    pub async fn complete_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        data: LiquidationCompletedData,
    ) -> Result<(), LiquidationLedgerError> {
        self.cala
            .post_transaction_in_op(
                db,
                CalaTransactionId::new(),
                templates::COMPLETE_LIQUIDATION,
                templates::CompleteLiquidationParams {
                    amount: data.sent_total,
                    journal_id: self.journal_id,
                    collateral_in_liquidation_account_id: data.collateral_in_liquidation_account_id,
                    liquidated_collateral_account_id: data.liquidated_collateral_account_id,
                    effective: crate::time::now().date_naive(),
                },
            )
            .await?;

        Ok(())
    }
}
