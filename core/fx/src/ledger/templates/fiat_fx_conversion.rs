use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::FX_TRANSACTION_ENTITY_TYPE};

pub const FIAT_FX_CONVERSION_VIA_TRADING_CODE: &str = "FIAT_FX_CONVERSION_VIA_TRADING";

#[derive(Debug)]
pub struct FiatFxConversionParams<S: std::fmt::Display> {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub source_currency: Currency,
    pub target_currency: Currency,
    pub source_amount: Decimal,
    pub target_amount: Decimal,
    pub source_account_id: cala_ledger::primitives::AccountId,
    pub target_account_id: cala_ledger::primitives::AccountId,
    pub trading_account_id: cala_ledger::primitives::AccountId,
    pub initiated_by: S,
    pub effective_date: chrono::NaiveDate,
}

impl<S: std::fmt::Display> FiatFxConversionParams<S> {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("source_currency")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("target_currency")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("source_amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("target_amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("source_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("target_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("trading_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("effective")
                .r#type(ParamDataType::Date)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("meta")
                .r#type(ParamDataType::Json)
                .build()
                .unwrap(),
        ]
    }
}

impl<S: std::fmt::Display> From<FiatFxConversionParams<S>> for Params {
    fn from(
        FiatFxConversionParams {
            entity_id,
            journal_id,
            source_currency,
            target_currency,
            source_amount,
            target_amount,
            source_account_id,
            target_account_id,
            trading_account_id,
            initiated_by,
            effective_date,
        }: FiatFxConversionParams<S>,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("source_currency", source_currency);
        params.insert("target_currency", target_currency);
        params.insert("source_amount", source_amount);
        params.insert("target_amount", target_amount);
        params.insert("source_account_id", source_account_id);
        params.insert("target_account_id", target_account_id);
        params.insert("trading_account_id", trading_account_id);
        params.insert("effective", effective_date);
        let entity_ref = chart_primitives::EntityRef::new(FX_TRANSACTION_ENTITY_TYPE, entity_id);
        params.insert(
            "meta",
            serde_json::json!({
                "entity_ref": entity_ref,
                "initiated_by": initiated_by.to_string(),
            }),
        );

        params
    }
}

pub struct FiatFxConversion;

impl FiatFxConversion {
    #[record_error_severity]
    #[instrument(name = "ledger.fiat_fx_conversion.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), FxLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Fiat FX conversion via trading account'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            // 1. Dr Trading Account — source_amount in source_currency
            NewTxTemplateEntry::builder()
                .entry_type("'FIAT_FX_CONVERSION_TRADING_DR_SOURCE'")
                .currency("params.source_currency")
                .account_id("params.trading_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.source_amount")
                .build()
                .expect("Couldn't build entry"),
            // 2. Cr Source Account — source_amount in source_currency
            NewTxTemplateEntry::builder()
                .entry_type("'FIAT_FX_CONVERSION_SOURCE_CR'")
                .currency("params.source_currency")
                .account_id("params.source_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.source_amount")
                .build()
                .expect("Couldn't build entry"),
            // 3. Dr Target Account — target_amount in target_currency
            NewTxTemplateEntry::builder()
                .entry_type("'FIAT_FX_CONVERSION_TARGET_DR'")
                .currency("params.target_currency")
                .account_id("params.target_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.target_amount")
                .build()
                .expect("Couldn't build entry"),
            // 4. Cr Trading Account — target_amount in target_currency
            NewTxTemplateEntry::builder()
                .entry_type("'FIAT_FX_CONVERSION_TRADING_CR_TARGET'")
                .currency("params.target_currency")
                .account_id("params.trading_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.target_amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = FiatFxConversionParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(FIAT_FX_CONVERSION_VIA_TRADING_CODE)
            .transaction(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build template");
        match ledger.tx_templates().create(template).await {
            Err(TxTemplateError::DuplicateCode(_)) => Ok(()),
            Err(e) => Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
}
