use chrono::NaiveDate;
use tracing::instrument;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, Currency, JournalId, TxTemplateId,
    tx_template::{
        NewTxTemplate, NewTxTemplateEntry, NewTxTemplateTransaction, error::TxTemplateError,
    },
    velocity::{NewParamDefinition, ParamDataType, Params},
};
use core_money::Satoshis;
use tracing_macros::record_error_severity;

use crate::collateral::ledger::CollateralLedgerError;

pub const SEND_COLLATERAL_TO_LIQUIDATION: &str = "SEND_COLLATERAL_TO_LIQUIDATION";

#[derive(Debug)]
pub struct SendCollateralToLiquidationParams {
    pub journal_id: JournalId,
    pub amount: Satoshis,
    pub collateral_account_id: CalaAccountId,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub effective: NaiveDate,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
}

impl SendCollateralToLiquidationParams {
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
                .name("collateral_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("collateral_in_liquidation_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("effective")
                .r#type(ParamDataType::Date)
                .build()
                .expect("Could not build param definition"),
            NewParamDefinition::builder()
                .name("meta")
                .r#type(ParamDataType::Json)
                .build()
                .unwrap(),
        ]
    }
}

impl From<SendCollateralToLiquidationParams> for Params {
    fn from(
        SendCollateralToLiquidationParams {
            journal_id,
            amount,
            collateral_account_id,
            collateral_in_liquidation_account_id,
            effective,
            initiated_by,
        }: SendCollateralToLiquidationParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", Currency::BTC);
        params.insert("amount", amount.to_btc());
        params.insert("collateral_account_id", collateral_account_id);
        params.insert(
            "collateral_in_liquidation_account_id",
            collateral_in_liquidation_account_id,
        );
        params.insert("effective", effective);
        params.insert(
            "meta",
            serde_json::json!({
                "initiated_by": initiated_by,
            }),
        );

        params
    }
}

pub struct SendCollateralToLiquidation;

impl SendCollateralToLiquidation {
    #[record_error_severity]
    #[instrument(name = "core_credit.collateral.ledger.templates.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CollateralLedgerError> {
        let transaction = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Send collateral to liquidation'")
            .build()
            .expect("Could not build new template transaction");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'SEND_COLLATERAL_TO_LIQUIDATION_DR'")
                .currency("params.currency")
                .account_id("params.collateral_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Could not build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'SEND_COLLATERAL_TO_LIQUIDATION_CR'")
                .currency("params.currency")
                .account_id("params.collateral_in_liquidation_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Could not build entry"),
        ];

        let params = SendCollateralToLiquidationParams::defs();

        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(SEND_COLLATERAL_TO_LIQUIDATION)
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
