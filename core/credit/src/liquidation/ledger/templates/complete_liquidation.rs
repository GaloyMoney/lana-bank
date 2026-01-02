use chrono::NaiveDate;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, Currency, JournalId, TxTemplateId,
    tx_template::{
        NewTxTemplate, NewTxTemplateEntry, NewTxTemplateTransaction, error::TxTemplateError,
    },
    velocity::{NewParamDefinition, ParamDataType, Params},
};
use core_money::Satoshis;

use crate::liquidation::ledger::LiquidationLedgerError;

pub const COMPLETE_LIQUIDATION: &str = "COMPLETE_LIQUIDATION";

#[derive(Debug)]
pub struct CompleteLiquidationParams {
    pub journal_id: JournalId,
    pub amount: Satoshis,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,
    pub effective: NaiveDate,
}

impl CompleteLiquidationParams {
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
                .name("collateral_in_liquidation_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("liquidated_collateral_account_id")
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

impl From<CompleteLiquidationParams> for Params {
    fn from(
        CompleteLiquidationParams {
            journal_id,
            amount,
            collateral_in_liquidation_account_id,
            liquidated_collateral_account_id,
            effective,
        }: CompleteLiquidationParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", Currency::BTC);
        params.insert("amount", amount.to_btc());
        params.insert(
            "collateral_in_liquidation_account_id",
            collateral_in_liquidation_account_id,
        );
        params.insert(
            "liquidated_collateral_account_id",
            liquidated_collateral_account_id,
        );
        params.insert("effective", effective);

        params
    }
}

pub struct CompleteLiquidation;

impl CompleteLiquidation {
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
                .entry_type("'COMPLETE_LIQUIDATION_DR'")
                .currency("params.currency")
                .account_id("params.collateral_in_liquidation_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Could not build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'COMPLETE_LIQUIDATION_CR'")
                .currency("params.currency")
                .account_id("params.liquidated_collateral_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Could not build entry"),
        ];

        let params = CompleteLiquidationParams::defs();

        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(COMPLETE_LIQUIDATION)
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
