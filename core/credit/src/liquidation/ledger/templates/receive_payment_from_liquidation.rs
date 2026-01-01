//! Template _Receive Payment From Liquidation_ records that (1) the
//! part of collateral that was in liquidation has already been
//! liquidated, and (2) fiat payment was received from liquidator.
//!
//! # Accounts in play
//!
//! - **fiat omnibus account**: source of all payments from liquidations
//! - **fiat facility holding account**: holds the payment from facility's liquidation until its allocation
//! - **btc in liquidation account**: tracks part of facility's collateral that is currently being liquidated
//! - **btc liquidated account**: tracks part of facility's collateral that has already been liquidated
use chrono::NaiveDate;
use tracing::instrument;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, Currency, JournalId, TxTemplateId,
    tx_template::{
        NewTxTemplate, NewTxTemplateEntry, NewTxTemplateTransaction, error::TxTemplateError,
    },
    velocity::{NewParamDefinition, ParamDataType, Params},
};
use core_money::{Satoshis, UsdCents};
use tracing_macros::record_error_severity;

use crate::{FacilityLiquidationInHoldingAccount, liquidation::ledger::LiquidationLedgerError};

pub const RECEIVE_PAYMENT_FROM_LIQUIDATION: &str = "RECEIVE_PAYMENT_FROM_LIQUIDATION";

#[derive(Debug)]
pub struct ReceivePaymentFromLiquidationParams {
    pub journal_id: JournalId,
    pub fiat_liquidation_omnibus_account_id: CalaAccountId,
    pub fiat_liquidation_in_holding_account_id: FacilityLiquidationInHoldingAccount,
    pub amount_received: UsdCents,
    pub currency: Currency,
    pub btc_in_liquidation_account_id: CalaAccountId,
    pub btc_liquidated_account_id: CalaAccountId,
    pub amount_liquidated: Satoshis,
    pub effective: NaiveDate,
}

impl ReceivePaymentFromLiquidationParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("omnibus_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("in_holding_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("amount_received")
                .r#type(ParamDataType::Decimal)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("currency")
                .r#type(ParamDataType::String)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("in_liquidation_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("liquidated_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("amount_liquidated")
                .r#type(ParamDataType::Decimal)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("effective")
                .r#type(ParamDataType::Date)
                .build()
                .expect("Could not build param definition"),
        ]
    }
}

impl From<ReceivePaymentFromLiquidationParams> for Params {
    fn from(
        ReceivePaymentFromLiquidationParams {
            journal_id,
            fiat_liquidation_omnibus_account_id,
            fiat_liquidation_in_holding_account_id,
            amount_received,
            currency,
            btc_in_liquidation_account_id,
            btc_liquidated_account_id,
            amount_liquidated,
            effective,
        }: ReceivePaymentFromLiquidationParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("omnibus_account_id", fiat_liquidation_omnibus_account_id);
        params.insert(
            "in_holding_account_id",
            fiat_liquidation_in_holding_account_id.into_inner(),
        );
        params.insert("amount_received", amount_received.to_usd());
        params.insert("currency", currency);
        params.insert("in_liquidation_account_id", btc_in_liquidation_account_id);
        params.insert("liquidated_account_id", btc_liquidated_account_id);
        params.insert("amount_liquidated", amount_liquidated.to_btc());
        params.insert("effective", effective);

        params
    }
}

pub struct ReceivePaymentFromLiquidation;

impl ReceivePaymentFromLiquidation {
    #[record_error_severity]
    #[instrument(name = "core_credit.liquidation.ledger.templates.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), LiquidationLedgerError> {
        let transaction = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .description("'Record received payment from liquidation and collateral liquidated'")
            .build()
            .expect("Could not build new template transaction");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RECEIVE_PAYMENT_FROM_LIQUIDATION_DR'")
                .currency("params.currency")
                .account_id("params.omnibus_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount_received")
                .build()
                .expect("Could not build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECEIVE_PAYMENT_FROM_LIQUIDATION_CR'")
                .currency("params.currency")
                .account_id("params.in_holding_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount_received")
                .build()
                .expect("Could not build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_COLLATERAL_LIQUIDATED_DR'")
                .currency("'BTC'")
                .account_id("params.in_liquidation_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount_liquidated")
                .build()
                .expect("Could not build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_COLLATERAL_LIQUIDATED_CR'")
                .currency("'BTC'")
                .account_id("params.liquidated_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount_liquidated")
                .build()
                .expect("Could not build entry"),
        ];

        let params = ReceivePaymentFromLiquidationParams::defs();

        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RECEIVE_PAYMENT_FROM_LIQUIDATION)
            .transaction(transaction)
            .entries(entries)
            .params(params)
            .build()
            .expect("Could not build transaction template");

        match ledger.tx_templates().create(template).await {
            Err(TxTemplateError::DuplicateCode) => Ok(()),
            Err(e) => Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
}
