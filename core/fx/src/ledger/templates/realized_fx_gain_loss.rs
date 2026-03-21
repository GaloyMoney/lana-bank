use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::FX_TRANSACTION_ENTITY_TYPE};

pub const REALIZED_FX_GAIN_LOSS_CODE: &str = "REALIZED_FX_GAIN_LOSS";

#[derive(Debug)]
pub struct RealizedFxGainLossParams<S: std::fmt::Display> {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub trading_account_id: cala_ledger::primitives::AccountId,
    pub gain_or_loss_account_id: cala_ledger::primitives::AccountId,
    /// "DEBIT" for gain (Dr Trading / Cr Gain), "CREDIT" for loss (Dr Loss / Cr Trading)
    pub trading_direction: &'static str,
    pub gain_loss_direction: &'static str,
    pub initiated_by: S,
    pub effective_date: chrono::NaiveDate,
}

impl<S: std::fmt::Display> RealizedFxGainLossParams<S> {
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
                .name("gain_or_loss_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("trading_direction")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("gain_loss_direction")
                .r#type(ParamDataType::String)
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

impl<S: std::fmt::Display> From<RealizedFxGainLossParams<S>> for Params {
    fn from(p: RealizedFxGainLossParams<S>) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", p.journal_id);
        params.insert("currency", p.currency);
        params.insert("amount", p.amount);
        params.insert("trading_account_id", p.trading_account_id);
        params.insert("gain_or_loss_account_id", p.gain_or_loss_account_id);
        params.insert("trading_direction", p.trading_direction);
        params.insert("gain_loss_direction", p.gain_loss_direction);
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

pub struct RealizedFxGainLoss;

impl RealizedFxGainLoss {
    #[record_error_severity]
    #[instrument(name = "ledger.realized_fx_gain_loss.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), FxLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Realized FX gain/loss adjustment'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'REALIZED_FX_GL_TRADING'")
                .currency("params.currency")
                .account_id("params.trading_account_id")
                .direction("params.trading_direction")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'REALIZED_FX_GL_ACCOUNT'")
                .currency("params.currency")
                .account_id("params.gain_or_loss_account_id")
                .direction("params.gain_loss_direction")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RealizedFxGainLossParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(REALIZED_FX_GAIN_LOSS_CODE)
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
