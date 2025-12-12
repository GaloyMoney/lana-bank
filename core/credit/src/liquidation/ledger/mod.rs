mod error;
mod templates;

use core_money::Satoshis;
use tracing::instrument;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, JournalId, TransactionId as CalaTransactionId,
};
use tracing_macros::record_error_severity;

pub use error::LiquidationLedgerError;

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
        db: es_entity::DbOp<'_>,
        tx_id: CalaTransactionId,
        amount: Satoshis,
        collateral_account_id: CalaAccountId,
        collateral_in_liquidation_account_id: CalaAccountId,
    ) -> Result<(), LiquidationLedgerError> {
        let mut db = self
            .cala
            .ledger_operation_from_db_op(db.with_db_time().await?);

        self.cala
            .post_transaction_in_op(
                &mut db,
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

        db.commit().await?;

        Ok(())
    }
}
