mod error;
mod templates;

use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, Currency, JournalId, TransactionId as CalaTransactionId,
};
use core_accounting::LedgerTransactionInitiator;
use core_money::Satoshis;
use es_entity::clock::ClockHandle;

pub use error::LiquidationLedgerError;

use super::RecordProceedsFromLiquidationData;

#[derive(Clone)]
pub struct LiquidationLedger {
    cala: CalaLedger,
    pub(crate) clock: ClockHandle,
    journal_id: JournalId,
}

impl LiquidationLedger {
    #[record_error_severity]
    #[instrument(name = "core_credit.liquidation.ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        clock: ClockHandle,
    ) -> Result<Self, LiquidationLedgerError> {
        templates::SendCollateralToLiquidation::init(cala).await?;
        templates::ReceiveProceedsFromLiquidation::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            clock,
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
        initiated_by: LedgerTransactionInitiator,
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
                    effective: self.clock.today(),
                    initiated_by,
                },
            )
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.liquidation.ledger.record_proceeds_from_liquidation_in_op",
        skip(self, db)
    )]
    pub async fn record_proceeds_from_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        tx_id: CalaTransactionId,
        data: RecordProceedsFromLiquidationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), LiquidationLedgerError> {
        self.cala
            .post_transaction_in_op(
                db,
                tx_id,
                templates::RECEIVE_PROCEEDS_FROM_LIQUIDATION,
                templates::ReceiveProceedsFromLiquidationParams {
                    journal_id: self.journal_id,
                    fiat_liquidation_proceeds_omnibus_account_id: data
                        .liquidation_proceeds_omnibus_account_id,
                    fiat_proceeds_from_liquidation_account_id: data
                        .proceeds_from_liquidation_account_id,
                    amount_received: data.amount_received,
                    currency: Currency::USD,
                    btc_in_liquidation_account_id: data.collateral_in_liquidation_account_id,
                    btc_liquidated_account_id: data.liquidated_collateral_account_id,
                    amount_liquidated: data.amount_liquidated,
                    effective: self.clock.today(),
                    initiated_by,
                },
            )
            .await?;

        Ok(())
    }
}
