use chrono::NaiveDate;
use derive_builder::Builder;
use rust_decimal::Decimal;

use cala_ledger::{
    AccountId as CalaAccountId, Currency, JournalId, TxTemplateId,
    primitives::DebitOrCredit,
    tx_template::{
        NewParamDefinition, NewTxTemplate, NewTxTemplateEntry, NewTxTemplateTransaction,
        ParamDataType, Params, error::TxTemplateError,
    },
};

use super::super::closing_metadata::AccountingClosingMetadata;

#[derive(Debug, Builder)]
pub struct EntryParams {
    pub account_id: CalaAccountId,
    pub currency: Currency,
    pub amount: Decimal,
    pub direction: DebitOrCredit,
}

impl EntryParams {
    pub fn builder() -> EntryParamsBuilder {
        EntryParamsBuilder::default()
    }

    pub fn populate_params(&self, params: &mut Params, n: usize) {
        params.insert(Self::account_id_param_name(n), self.account_id);
        params.insert(Self::currency_param_name(n), self.currency);
        params.insert(Self::amount_param_name(n), self.amount);
        params.insert(Self::direction_param_name(n), self.direction);
    }

    fn defs_for_entry(n: usize) -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name(Self::account_id_param_name(n))
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name(Self::currency_param_name(n))
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name(Self::amount_param_name(n))
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name(Self::direction_param_name(n))
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name(Self::layer_param_name(n))
                .r#type(ParamDataType::String)
                .default_expr("SETTLED")
                .build()
                .unwrap(),
        ]
    }

    pub fn account_id_param_name(n: usize) -> String {
        format!("entry_{n}_account_id")
    }

    pub fn currency_param_name(n: usize) -> String {
        format!("entry_{n}_currency")
    }

    pub fn amount_param_name(n: usize) -> String {
        format!("entry_{n}_amount")
    }

    pub fn direction_param_name(n: usize) -> String {
        format!("entry_{n}_direction")
    }

    pub fn layer_param_name(n: usize) -> String {
        format!("entry_{n}_layer")
    }
}

#[derive(Debug)]
pub struct ClosingTransactionParams<S: std::fmt::Display> {
    pub journal_id: JournalId,
    pub description: String,
    pub effective: chrono::NaiveDate,
    pub entries_params: Vec<EntryParams>,
    pub initiated_by: S,
}

impl<S: std::fmt::Display> From<ClosingTransactionParams<S>> for Params {
    fn from(input_params: ClosingTransactionParams<S>) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", input_params.journal_id);
        params.insert("description", input_params.description);
        params.insert("effective", input_params.effective);
        let mut meta = AccountingClosingMetadata::closing_tx_metadata_json();
        meta["initiated_by"] = serde_json::Value::String(input_params.initiated_by.to_string());
        params.insert("meta", meta);

        for (n, entry) in input_params.entries_params.into_iter().enumerate() {
            entry.populate_params(&mut params, n);
        }

        params
    }
}

impl<S: std::fmt::Display> ClosingTransactionParams<S> {
    pub fn new(
        journal_id: JournalId,
        description: String,
        effective: NaiveDate,
        entries_params: Vec<EntryParams>,
        initiated_by: S,
    ) -> ClosingTransactionParams<S> {
        Self {
            journal_id,
            description,
            effective,
            entries_params,
            initiated_by,
        }
    }

    pub fn defs(n: usize) -> Vec<NewParamDefinition> {
        let mut params = vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("description")
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
        ];
        for i in 0..n {
            params.extend(EntryParams::defs_for_entry(i));
        }
        params
    }

    pub fn template_code(&self) -> String {
        format!("CLOSING_TRANSACTION_{}", self.description)
    }

    pub fn tx_entry_type(&self, i: usize) -> String {
        format!("'CLOSING_TRANSACTION_{}_ENTRY_{}'", self.description, i)
    }
}

pub async fn find_or_create_template_in_op<S: std::fmt::Display>(
    op: &mut es_entity::DbOp<'_>,
    cala: &cala_ledger::CalaLedger,
    params: &ClosingTransactionParams<S>,
) -> Result<String, TxTemplateError> {
    let n_entries = params.entries_params.len();
    let code = params.template_code();

    let mut entries = vec![];
    for i in 0..n_entries {
        entries.push(
            NewTxTemplateEntry::builder()
                .entry_type(params.tx_entry_type(i))
                .account_id(format!("params.{}", EntryParams::account_id_param_name(i)))
                .units(format!("params.{}", EntryParams::amount_param_name(i)))
                .currency(format!("params.{}", EntryParams::currency_param_name(i)))
                .layer(format!("params.{}", EntryParams::layer_param_name(i)))
                .direction(format!("params.{}", EntryParams::direction_param_name(i)))
                .build()
                .expect("Couldn't build entry for ClosingTransactionTemplate"),
        );
    }

    let tx_input = NewTxTemplateTransaction::builder()
        .journal_id("params.journal_id")
        .description("params.description")
        .effective("params.effective")
        .metadata("params.meta")
        .build()
        .expect("Couldn't build TxInput for ClosingTransactionTemplate");

    let params = ClosingTransactionParams::<String>::defs(n_entries);
    let new_template = NewTxTemplate::builder()
        .id(TxTemplateId::new())
        .code(&code)
        .transaction(tx_input)
        .entries(entries)
        .params(params)
        .description(format!(
            "Template to execute a closing transaction with {} entries.",
            n_entries
        ))
        .build()
        .expect("Couldn't build template for ClosingTransactionTemplate");
    match cala.tx_templates().create_in_op(op, new_template).await {
        Err(TxTemplateError::DuplicateCode) => Ok(code),
        Err(e) => Err(e),
        Ok(template) => Ok(template.into_values().code),
    }
}
