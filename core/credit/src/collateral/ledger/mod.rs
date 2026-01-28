mod error;
pub mod templates;

use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{CalaLedger, Currency, JournalId, TransactionId as CalaTransactionId};
use core_accounting::LedgerTransactionInitiator;
use core_money::Satoshis;
use es_entity::clock::ClockHandle;

pub use error::CollateralLedgerError;

use crate::primitives::{
    CalaAccountId, CollateralAction, CollateralUpdate, LedgerOmnibusAccountIds,
};

#[derive(Clone)]
pub struct CollateralLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    clock: ClockHandle,
    collateral_omnibus_account_ids: LedgerOmnibusAccountIds,
    btc: Currency,
}

impl CollateralLedger {
    #[record_error_severity]
    #[instrument(name = "core_credit.collateral.ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        clock: ClockHandle,
        collateral_omnibus_account_ids: LedgerOmnibusAccountIds,
    ) -> Result<Self, CollateralLedgerError> {
        templates::AddCollateral::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;
        templates::SendCollateralToLiquidation::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            clock,
            collateral_omnibus_account_ids,
            btc: Currency::BTC,
        })
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.collateral.ledger.update_collateral_amount",
        skip(self, op)
    )]
    pub async fn update_collateral_amount(
        &self,
        op: &mut es_entity::DbOp<'_>,
        CollateralUpdate {
            tx_id,
            abs_diff,
            action,
            effective,
        }: CollateralUpdate,
        collateral_account_id: CalaAccountId,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollateralLedgerError> {
        match action {
            CollateralAction::Add => {
                self.cala
                    .post_transaction_in_op(
                        op,
                        tx_id,
                        templates::ADD_COLLATERAL_CODE,
                        templates::AddCollateralParams {
                            journal_id: self.journal_id,
                            currency: self.btc,
                            amount: abs_diff.to_btc(),
                            collateral_account_id,
                            bank_collateral_account_id: self
                                .collateral_omnibus_account_ids
                                .account_id,
                            effective,
                            initiated_by,
                        },
                    )
                    .await
            }
            CollateralAction::Remove => {
                self.cala
                    .post_transaction_in_op(
                        op,
                        tx_id,
                        templates::REMOVE_COLLATERAL_CODE,
                        templates::RemoveCollateralParams {
                            journal_id: self.journal_id,
                            currency: self.btc,
                            amount: abs_diff.to_btc(),
                            collateral_account_id,
                            bank_collateral_account_id: self
                                .collateral_omnibus_account_ids
                                .account_id,
                            effective,
                            initiated_by,
                        },
                    )
                    .await
            }
        }?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.collateral.ledger.record_collateral_sent_to_liquidation",
        skip(self, db)
    )]
    pub async fn record_collateral_sent_to_liquidation(
        &self,
        db: &mut es_entity::DbOp<'_>,
        tx_id: CalaTransactionId,
        amount: Satoshis,
        collateral_account_id: CalaAccountId,
        collateral_in_liquidation_account_id: CalaAccountId,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollateralLedgerError> {
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
}
