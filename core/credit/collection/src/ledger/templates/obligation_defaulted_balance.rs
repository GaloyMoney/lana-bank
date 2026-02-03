use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::primitives::CalaAccountId;

use super::super::error::CollectionLedgerError;

pub const RECORD_OBLIGATION_DEFAULTED_BALANCE_CODE: &str = "RECORD_OBLIGATION_DEFAULTED_BALANCE";

#[derive(Debug)]
pub struct RecordObligationDefaultedBalanceParams {
    pub journal_id: JournalId,
    pub amount: Decimal,
    pub receivable_account_id: CalaAccountId,
    pub defaulted_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
}

impl RecordObligationDefaultedBalanceParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("receivable_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("defaulted_account_id")
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
impl From<RecordObligationDefaultedBalanceParams> for Params {
    fn from(
        RecordObligationDefaultedBalanceParams {
            journal_id,
            amount,
            receivable_account_id,
            defaulted_account_id,
            effective,
            initiated_by,
        }: RecordObligationDefaultedBalanceParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("amount", amount);
        params.insert("receivable_account_id", receivable_account_id);
        params.insert("defaulted_account_id", defaulted_account_id);
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

pub struct RecordObligationDefaultedBalance;

impl RecordObligationDefaultedBalance {
    #[record_error_severity]
    #[instrument(
        name = "collection_ledger.record_obligation_defaulted_balance.init",
        skip_all
    )]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CollectionLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Record a defaulted obligation balance'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_OBLIGATION_DEFAULTED_BALANCE_CR'")
                .currency("'USD'")
                .account_id("params.receivable_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_OBLIGATION_DEFAULTED_BALANCE_DR'")
                .currency("'USD'")
                .account_id("params.defaulted_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RecordObligationDefaultedBalanceParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RECORD_OBLIGATION_DEFAULTED_BALANCE_CODE)
            .transaction(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build template");
        match ledger.tx_templates().create(template).await {
            Err(TxTemplateError::DuplicateCode) => Ok(()),
            Err(e) => Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
}
