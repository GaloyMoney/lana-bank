use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::FX_TRANSACTION_ENTITY_TYPE};

pub const FX_ROUNDING_ADJUSTMENT_CODE: &str = "FX_ROUNDING_ADJUSTMENT";

#[derive(Debug)]
pub struct FxRoundingAdjustmentParams<S: std::fmt::Display> {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub trading_account_id: cala_ledger::primitives::AccountId,
    pub rounding_account_id: cala_ledger::primitives::AccountId,
    pub initiated_by: S,
    pub effective_date: chrono::NaiveDate,
}

impl<S: std::fmt::Display> FxRoundingAdjustmentParams<S> {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("currency")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("trading_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("rounding_account_id")
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

impl<S: std::fmt::Display> From<FxRoundingAdjustmentParams<S>> for Params {
    fn from(p: FxRoundingAdjustmentParams<S>) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", p.journal_id);
        params.insert("currency", p.currency);
        params.insert("amount", p.amount);
        params.insert("trading_account_id", p.trading_account_id);
        params.insert("rounding_account_id", p.rounding_account_id);
        params.insert("effective", p.effective_date);
        let entity_ref = chart_primitives::EntityRef::new(FX_TRANSACTION_ENTITY_TYPE, p.entity_id);
        params.insert(
            "meta",
            serde_json::json!({
                "entity_ref": entity_ref,
                "initiated_by": p.initiated_by.to_string(),
            }),
        );
        params
    }
}

pub struct FxRoundingAdjustment;

impl FxRoundingAdjustment {
    #[record_error_severity]
    #[instrument(name = "ledger.fx_rounding_adjustment.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), FxLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'FX rounding adjustment'")
            .build()
            .expect("Couldn't build TxInput");
        // Rounding always goes: Dr Rounding account / Cr Trading account
        // (bank keeps the sub-cent difference)
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'FX_ROUNDING_DR_ROUNDING'")
                .currency("params.currency")
                .account_id("params.rounding_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'FX_ROUNDING_CR_TRADING'")
                .currency("params.currency")
                .account_id("params.trading_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = FxRoundingAdjustmentParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(FX_ROUNDING_ADJUSTMENT_CODE)
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
