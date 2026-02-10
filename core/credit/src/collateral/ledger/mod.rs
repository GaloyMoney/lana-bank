mod accounts;
mod error;
pub mod templates;

use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    CalaLedger, Currency, JournalId, TransactionId as CalaTransactionId, account::NewAccount,
};
use core_accounting::{EntityRef, LedgerTransactionInitiator};
use es_entity::clock::ClockHandle;
use money::Satoshis;

pub use accounts::{CollateralLedgerAccountIds, LiquidationProceedsAccountIds};
pub use error::CollateralLedgerError;

use crate::{
    ledger::LiquidationAccountSets,
    primitives::{
        COLLATERAL_ENTITY_TYPE, CalaAccountId, CollateralDirection, CollateralId, CollateralUpdate,
        LedgerOmnibusAccountIds,
    },
};

use super::RecordProceedsFromLiquidationData;

#[derive(Clone)]
pub struct CollateralLedger {
    cala: CalaLedger,
    journal_id: JournalId,
    clock: ClockHandle,
    collateral_omnibus_account_ids: LedgerOmnibusAccountIds,
    liquidation_account_sets: LiquidationAccountSets,
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
        liquidation_account_sets: LiquidationAccountSets,
    ) -> Result<Self, CollateralLedgerError> {
        templates::AddCollateral::init(cala).await?;
        templates::RemoveCollateral::init(cala).await?;
        templates::SendCollateralToLiquidation::init(cala).await?;
        templates::ReceiveProceedsFromLiquidation::init(cala).await?;

        Ok(Self {
            cala: cala.clone(),
            journal_id,
            clock,
            collateral_omnibus_account_ids,
            liquidation_account_sets,
            btc: Currency::BTC,
        })
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.collateral.ledger.create_collateral_accounts_in_op",
        skip(self, op)
    )]
    pub async fn create_collateral_accounts_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        CollateralLedgerAccountIds {
            collateral_in_liquidation_account_id,
            liquidated_collateral_account_id,

            collateral_account_id: _,
            liquidation_proceeds_omnibus_account_id: _,
            facility_proceeds_from_liquidation_account_id: _,
            facility_uncovered_outstanding_account_id: _,
            facility_payment_holding_account_id: _,
        }: CollateralLedgerAccountIds,
    ) -> Result<(), CollateralLedgerError> {
        let entity_ref = EntityRef::new(COLLATERAL_ENTITY_TYPE, collateral_id);

        let collateral_in_liquidation_reference =
            &format!("collateral-in-liquidation:{collateral_id}");
        let collateral_in_liquidation_name =
            &format!("Collateral in Liquidation Account for Collateral {collateral_id}");
        self.create_account_in_op(
            op,
            collateral_in_liquidation_account_id,
            self.liquidation_account_sets.collateral_in_liquidation,
            collateral_in_liquidation_reference,
            collateral_in_liquidation_name,
            collateral_in_liquidation_name,
            entity_ref.clone(),
        )
        .await?;

        let liquidated_collateral_reference = &format!("liquidated-collateral:{collateral_id}");
        let liquidated_collateral_name =
            &format!("Liquidated Collateral Account for Collateral {collateral_id}");
        self.create_account_in_op(
            op,
            liquidated_collateral_account_id,
            self.liquidation_account_sets.liquidated_collateral,
            liquidated_collateral_reference,
            liquidated_collateral_name,
            liquidated_collateral_name,
            entity_ref,
        )
        .await?;

        Ok(())
    }

    async fn create_account_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        id: impl Into<CalaAccountId>,
        parent_account_set: crate::ledger::InternalAccountSetDetails,
        reference: &str,
        name: &str,
        description: &str,
        entity_ref: EntityRef,
    ) -> Result<(), CollateralLedgerError> {
        let id = id.into();

        let new_ledger_account = NewAccount::builder()
            .id(id)
            .external_id(reference)
            .name(name)
            .description(description)
            .code(id.to_string())
            .normal_balance_type(parent_account_set.normal_balance_type())
            .metadata(serde_json::json!({"entity_ref": entity_ref}))
            .expect("Could not add metadata")
            .build()
            .expect("Could not build new account");
        let ledger_account = self
            .cala
            .accounts()
            .create_in_op(op, new_ledger_account)
            .await?;
        self.cala
            .account_sets()
            .add_member_in_op(op, parent_account_set.id(), ledger_account.id)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "core_credit.collateral.ledger.update_collateral_amount_in_op",
        skip(self, op)
    )]
    pub async fn update_collateral_amount_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        CollateralUpdate {
            tx_id,
            abs_diff,
            direction,
            effective,
        }: CollateralUpdate,
        collateral_account_id: CalaAccountId,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollateralLedgerError> {
        match direction {
            CollateralDirection::Add => {
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
            CollateralDirection::Remove => {
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
        name = "core_credit.collateral.ledger.record_collateral_sent_to_liquidation_in_op",
        skip(self, db)
    )]
    pub async fn record_collateral_sent_to_liquidation_in_op(
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

    #[record_error_severity]
    #[instrument(
        name = "core_credit.collateral.ledger.record_proceeds_from_liquidation_in_op",
        skip(self, db)
    )]
    pub async fn record_proceeds_from_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        data: RecordProceedsFromLiquidationData,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CollateralLedgerError> {
        self.cala
            .post_transaction_in_op(
                db,
                data.ledger_tx_id,
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
