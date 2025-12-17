use chrono::NaiveDate;
use derive_builder::Builder;
use rust_decimal::Decimal;

use cala_ledger::{
    AccountId as CalaAccountId, Currency, JournalId,
    primitives::DebitOrCredit,
    tx_template::{NewParamDefinition, ParamDataType, Params},
};

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
pub(super) struct ClosingTransactionParams {
    pub(super) journal_id: JournalId,
    pub(super) description: String,
    pub(super) effective: chrono::NaiveDate,
    pub(super) entries_params: Vec<EntryParams>,
}

impl From<ClosingTransactionParams> for Params {
    fn from(input_params: ClosingTransactionParams) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", input_params.journal_id);
        params.insert("description", input_params.description);
        params.insert("effective", input_params.effective);

        for (n, entry) in input_params.entries_params.into_iter().enumerate() {
            entry.populate_params(&mut params, n);
        }

        params
    }
}

impl ClosingTransactionParams {
    pub(super) fn new(
        journal_id: JournalId,
        description: String,
        effective: NaiveDate,
        entries_params: Vec<EntryParams>,
    ) -> ClosingTransactionParams {
        Self {
            journal_id,
            description,
            effective,
            entries_params,
        }
    }

    pub(super) fn defs(n: usize) -> Vec<NewParamDefinition> {
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
        ];
        for i in 0..n {
            params.extend(EntryParams::defs_for_entry(i));
        }
        params
    }

    pub(super) fn template_code(&self) -> String {
        format!("CLOSING_TRANSACTION_{}", self.description)
    }

    pub(super) fn tx_entry_type(&self, i: usize) -> String {
        format!("'CLOSING_TRANSACTION_{}_ENTRY_{}'", self.description, i)
    }
}
