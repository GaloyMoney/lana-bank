mod error;
mod templates;

use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{CalaLedger, Currency, JournalId};
use core_accounting::LedgerTransactionInitiator;

pub use error::CollateralLedgerError;

use crate::{
    ledger::PendingCreditFacilityAccountIds,
    primitives::{CalaAccountId, CollateralAction, CollateralUpdate, LedgerOmnibusAccountIds},
};

#[derive(Clone)]
pub struct CollateralLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    collateral_omnibus_account_ids: LedgerOmnibusAccountIds,
    btc: Currency,
}

impl CollateralLedger {
    #[record_error_severity]
    #[instrument(name = "core_credit.collateral.ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        collateral_omnibus_account_ids: LedgerOmnibusAccountIds,
    ) -> Result<Self, CollateralLedgerError> {
        templates::AddCollateral::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            collateral_omnibus_account_ids,
            btc: Currency::BTC,
        })
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.collateral.ledger.update_pending_credit_facility_collateral",
        skip(self, op)
    )]
    pub async fn update_pending_credit_facility_collateral(
        &self,
        op: &mut es_entity::DbOp<'_>,
        CollateralUpdate {
            tx_id,
            abs_diff,
            action,
            effective,
        }: CollateralUpdate,
        pending_credit_facility_account_ids: PendingCreditFacilityAccountIds,
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
                            collateral_account_id: pending_credit_facility_account_ids
                                .collateral_account_id,
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
                            collateral_account_id: pending_credit_facility_account_ids
                                .collateral_account_id,
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
        name = "core_credit.collateral.ledger.update_credit_facility_collateral",
        skip(self, op)
    )]
    pub async fn update_credit_facility_collateral(
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
}
