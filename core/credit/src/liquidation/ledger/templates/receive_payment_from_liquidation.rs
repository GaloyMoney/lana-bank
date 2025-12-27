use chrono::NaiveDate;
use tracing::instrument;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, Currency, JournalId, TxTemplateId,
    tx_template::{
        NewTxTemplate, NewTxTemplateEntry, NewTxTemplateTransaction, error::TxTemplateError,
    },
    velocity::{NewParamDefinition, ParamDataType, Params},
};
use core_money::UsdCents;
use tracing_macros::record_error_severity;

use crate::liquidation::ledger::LiquidationLedgerError;

pub const RECEIVE_PAYMENT_FROM_LIQUIDATION: &str = "RECEIVE_PAYMENT_FROM_LIQUIDATION";

#[derive(Debug)]
pub struct ReceivePaymentFromLiquidationParams {
    pub journal_id: JournalId,
    pub amount: UsdCents,
    pub currency: Currency,
    pub omnibus_account_id: CalaAccountId,
    pub receivable_account_id: CalaAccountId,
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
                .name("amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("currency")
                .r#type(ParamDataType::String)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("omnibus_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("receivable_account_id")
                .r#type(ParamDataType::Uuid)
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
            amount,
            currency,
            receivable_account_id,
            journal_id,
            effective,
            omnibus_account_id,
        }: ReceivePaymentFromLiquidationParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount.to_usd());
        params.insert("omnibus_account_id", omnibus_account_id);
        params.insert("receivable_account_id", receivable_account_id);
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
            .description("'Send collateral to liquidation'")
            .build()
            .expect("Could not build new template transaction");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RECEIVE_PAYMENT_FROM_LIQUIDATION_DR'")
                .currency("params.currency")
                .account_id("params.omnibus_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Could not build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECEIVE_PAYMENT_FROM_LIQUIDATION_CR'")
                .currency("params.currency")
                .account_id("params.receivable_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
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
