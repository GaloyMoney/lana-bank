use derive_builder::Builder;
use rust_decimal::Decimal;

use cala_ledger::{
    AccountId as CalaAccountId,
    primitives::DebitOrCredit,
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

#[derive(Debug, Builder)]
pub struct EntryParams {
    pub account_id: CalaAccountId,
    pub currency: Currency,
    pub amount: Decimal,
    pub description: String,
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
        params.insert(Self::description_param_name(n), self.description.clone());
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
                .name(Self::description_param_name(n))
                .r#type(ParamDataType::String)
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

    fn account_id_param_name(n: usize) -> String {
        format!("entry_{n}_account_id")
    }

    fn currency_param_name(n: usize) -> String {
        format!("entry_{n}_currency")
    }

    fn amount_param_name(n: usize) -> String {
        format!("entry_{n}_amount")
    }

    fn description_param_name(n: usize) -> String {
        format!("entry_{n}_description")
    }

    fn direction_param_name(n: usize) -> String {
        format!("entry_{n}_direction")
    }

    fn layer_param_name(n: usize) -> String {
        format!("entry_{n}_layer")
    }
}

#[derive(Debug)]
pub struct AnnualClosingTransactionParams {
    pub journal_id: JournalId,
    pub description: String,
    pub effective: chrono::NaiveDate,
    pub entry_params: Vec<EntryParams>,
}

impl From<AnnualClosingTransactionParams> for Params {
    fn from(input_params: AnnualClosingTransactionParams) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", input_params.journal_id);
        params.insert("description", input_params.description);
        params.insert("effective", input_params.effective);

        for (n, entry) in input_params.entry_params.iter().enumerate() {
            entry.populate_params(&mut params, n);
        }

        params
    }
}

impl AnnualClosingTransactionParams {
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
        ];
        for i in 0..n {
            params.extend(EntryParams::defs_for_entry(i));
        }
        params
    }
}

pub(super) struct AnnualClosingTransactionTemplate {
    pub n_entries: usize,
}

impl AnnualClosingTransactionTemplate {
    pub fn code(&self) -> String {
        format!("ANNUAL_CLOSING_TRANSACTION_{}", self.n_entries)
    }

    pub async fn init(ledger: &CalaLedger, n_entries: usize) -> Result<Self, TxTemplateError> {
        let res = Self { n_entries };
        res.find_or_create_template(ledger).await?;
        Ok(res)
    }

    async fn find_or_create_template(&self, ledger: &CalaLedger) -> Result<(), TxTemplateError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .description("params.description")
            .effective("params.effective")
            .build()
            .expect("Couldn't build TxInput for AnnualClosingTransactionTxTemplate");

        let params = AnnualClosingTransactionParams::defs(self.n_entries);
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(self.code())
            .transaction(tx_input)
            .entries(self.entries())
            .params(params)
            .description(format!(
                "Template to execute an annual closing transaction with {} entries.",
                self.n_entries
            ))
            .build()
            .expect("Couldn't build template for AnnualClosingTransactionTxTemplate");
        match ledger.tx_templates().create(template).await {
            Err(TxTemplateError::DuplicateCode) => Ok(()),
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }

    fn entries(&self) -> Vec<NewTxTemplateEntry> {
        let mut entries = vec![];
        for i in 0..self.n_entries {
            entries.push(
                NewTxTemplateEntry::builder()
                    .entry_type(format!(
                        "'ANNUAL_CLOSING_TRANSACTION_{}_ENTRY_{}'",
                        self.n_entries, i
                    ))
                    .account_id(format!("params.{}", EntryParams::account_id_param_name(i)))
                    .units(format!("params.{}", EntryParams::amount_param_name(i)))
                    .currency(format!("params.{}", EntryParams::currency_param_name(i)))
                    .layer(format!("params.{}", EntryParams::layer_param_name(i)))
                    .direction(format!("params.{}", EntryParams::direction_param_name(i)))
                    .description(format!("params.entry_{i}_description"))
                    .build()
                    .expect("Couldn't build AnnualClosingTransaction TxTemplateEntry entry"),
            );
        }
        entries
    }
}