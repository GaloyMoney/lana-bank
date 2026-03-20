use core_price::ExchangeRateMetadata;
use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{
    ledger::error::*,
    primitives::{CalaAccountId, DEPOSIT_TRANSACTION_ENTITY_TYPE},
};

pub const RECORD_DEPOSIT_CODE: &str = "RECORD_DEPOSIT";

#[derive(Debug)]
pub struct RecordDepositParams<S: std::fmt::Display> {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub exchange_rate_metadata: ExchangeRateMetadata,
    pub deposit_omnibus_account_id: CalaAccountId,
    pub credit_account_id: CalaAccountId,
    pub initiated_by: S,
    pub effective_date: chrono::NaiveDate,
}

impl<S: std::fmt::Display> RecordDepositParams<S> {
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
                .name("base_currency")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("quote_currency")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("rate_type")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("base_currency_value")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("reference_rate")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("exchange_rate_timestamp")
                .r#type(ParamDataType::Timestamp)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("deposit_omnibus_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_account_id")
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

impl<S: std::fmt::Display> From<RecordDepositParams<S>> for Params {
    fn from(
        RecordDepositParams {
            entity_id,
            journal_id,
            currency,
            amount,
            exchange_rate_metadata,
            deposit_omnibus_account_id,
            credit_account_id,
            initiated_by,
            effective_date,
        }: RecordDepositParams<S>,
    ) -> Self {
        let ExchangeRateMetadata {
            base_currency,
            quote_currency,
            rate_type,
            reference_rate,
            exchange_rate_timestamp,
            base_currency_value,
        } = exchange_rate_metadata;
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("base_currency", base_currency.iso());
        params.insert("quote_currency", quote_currency.iso());
        params.insert("rate_type", rate_type.as_str());
        params.insert("base_currency_value", base_currency_value.to_major());
        params.insert("reference_rate", reference_rate);
        params.insert("exchange_rate_timestamp", exchange_rate_timestamp);
        params.insert("deposit_omnibus_account_id", deposit_omnibus_account_id);
        params.insert("credit_account_id", credit_account_id);
        params.insert("effective", effective_date);
        let entity_ref =
            chart_primitives::EntityRef::new(DEPOSIT_TRANSACTION_ENTITY_TYPE, entity_id);
        params.insert(
            "meta",
            serde_json::json!({
                "entity_ref": entity_ref,
                "initiated_by": initiated_by.to_string(),
                "rate": rate_metadata(
                    base_currency,
                    quote_currency,
                    rate_type,
                    base_currency_value,
                    reference_rate,
                    exchange_rate_timestamp,
                ),
            }),
        );

        params
    }
}

fn rate_metadata(
    base_currency: money::CurrencyCode,
    quote_currency: money::CurrencyCode,
    rate_type: core_price::ExchangeRateType,
    base_currency_value: money::Amount,
    reference_rate: Decimal,
    exchange_rate_timestamp: chrono::DateTime<chrono::Utc>,
) -> serde_json::Value {
    serde_json::json!({
        "base_currency": base_currency.iso(),
        "quote_currency": quote_currency.iso(),
        "type": rate_type.as_str(),
        "base_currency_value": base_currency_value.to_major(),
        "reference_rate": reference_rate,
        "exchange_rate_timestamp": exchange_rate_timestamp,
    })
}

pub struct RecordDeposit;

impl RecordDeposit {
    #[record_error_severity]
    #[instrument(name = "ledger.record_deposit.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), DepositLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Record a deposit'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_DEPOSIT_DR'")
                .currency("params.currency")
                .account_id("params.deposit_omnibus_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_DEPOSIT_CR'")
                .currency("params.currency")
                .account_id("params.credit_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RecordDepositParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RECORD_DEPOSIT_CODE)
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use rust_decimal_macros::dec;

    use super::*;
    use money::{Amount as MoneyAmount, CurrencyCode, UsdCents};

    #[test]
    fn record_deposit_defs_include_rate_fields() {
        let defs = RecordDepositParams::<String>::defs();
        let defs = serde_json::to_value(defs).expect("defs should serialize");
        let names: Vec<_> = defs
            .as_array()
            .expect("defs should be array")
            .iter()
            .filter_map(|d| d.get("name"))
            .filter_map(|name| name.as_str())
            .collect();

        assert!(names.contains(&"base_currency"));
        assert!(names.contains(&"quote_currency"));
        assert!(names.contains(&"rate_type"));
        assert!(names.contains(&"base_currency_value"));
        assert!(names.contains(&"reference_rate"));
        assert!(names.contains(&"exchange_rate_timestamp"));
    }

    #[test]
    fn rate_metadata_contains_exchange_rate_context() {
        let timestamp = Utc.with_ymd_and_hms(2026, 3, 20, 12, 0, 0).unwrap();
        let meta = rate_metadata(
            CurrencyCode::USD,
            CurrencyCode::USD,
            core_price::ExchangeRateType::Spot,
            MoneyAmount::from(UsdCents::from(10_000)),
            dec!(1),
            timestamp,
        );

        assert_eq!(
            meta,
            serde_json::json!({
                "base_currency": "USD",
                "quote_currency": "USD",
                "type": "SPOT",
                "base_currency_value": dec!(100),
                "reference_rate": dec!(1),
                "exchange_rate_timestamp": timestamp,
            }),
        );
    }
}
