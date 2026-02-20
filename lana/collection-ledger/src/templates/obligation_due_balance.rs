use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use core_credit_collection::CalaAccountId;

use core_credit_collection::CollectionLedgerError;

pub const RECORD_OBLIGATION_DUE_BALANCE_CODE: &str = "RECORD_OBLIGATION_DUE_BALANCE";

#[derive(Debug)]
pub struct RecordObligationDueBalanceParams<S: std::fmt::Display> {
    pub journal_id: JournalId,
    pub amount: Decimal,
    pub receivable_not_yet_due_account_id: CalaAccountId,
    pub receivable_due_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
    pub initiated_by: S,
}

impl<S: std::fmt::Display> RecordObligationDueBalanceParams<S> {
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
                .name("receivable_not_yet_due_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("receivable_due_account_id")
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
impl<S: std::fmt::Display> From<RecordObligationDueBalanceParams<S>> for Params {
    fn from(
        RecordObligationDueBalanceParams {
            journal_id,
            amount,
            receivable_not_yet_due_account_id,
            receivable_due_account_id,
            effective,
            initiated_by,
        }: RecordObligationDueBalanceParams<S>,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("amount", amount);
        params.insert(
            "receivable_not_yet_due_account_id",
            receivable_not_yet_due_account_id,
        );
        params.insert("receivable_due_account_id", receivable_due_account_id);
        params.insert("effective", effective);
        params.insert(
            "meta",
            serde_json::json!({
                "initiated_by": initiated_by.to_string(),
            }),
        );

        params
    }
}

pub struct RecordObligationDueBalance;

impl RecordObligationDueBalance {
    #[record_error_severity]
    #[instrument(
        name = "collection.ledger.record_obligation_due_balance.init",
        skip_all
    )]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CollectionLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Record a due obligation balance'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_OBLIGATION_DUE_BALANCE_CR'")
                .currency("'USD'")
                .account_id("params.receivable_not_yet_due_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_OBLIGATION_DUE_BALANCE_DR'")
                .currency("'USD'")
                .account_id("params.receivable_due_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RecordObligationDueBalanceParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RECORD_OBLIGATION_DUE_BALANCE_CODE)
            .transaction(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build template");
        match ledger.tx_templates().create(template).await {
            Err(TxTemplateError::DuplicateCode) => Ok(()),
            Err(e) => Err(CollectionLedgerError::from_ledger(e)),
            Ok(_) => Ok(()),
        }
    }
}
