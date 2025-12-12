use chrono::NaiveDate;
use derive_builder::Builder;
use rust_decimal::Decimal;

use cala_ledger::{
    AccountId as CalaAccountId,
    primitives::DebitOrCredit,
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use super::closing_balances::ClosingTxEntry;

#[derive(Debug, Builder)]
pub struct EntryParams {
    pub account_id: CalaAccountId,
    pub currency: Currency,
    pub amount: Decimal,
    pub direction: DebitOrCredit,
}

impl From<ClosingTxEntry> for EntryParams {
    fn from(spec: ClosingTxEntry) -> Self {
        EntryParams::builder()
            .account_id(spec.account_id.into())
            .amount(spec.amount)
            .currency(spec.currency)
            .direction(spec.direction)
            .build()
            .expect("Failed to build EntryParams from ClosingTxEntry")
    }
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

    fn account_id_param_name(n: usize) -> String {
        format!("entry_{n}_account_id")
    }

    fn currency_param_name(n: usize) -> String {
        format!("entry_{n}_currency")
    }

    fn amount_param_name(n: usize) -> String {
        format!("entry_{n}_amount")
    }

    fn direction_param_name(n: usize) -> String {
        format!("entry_{n}_direction")
    }

    fn layer_param_name(n: usize) -> String {
        format!("entry_{n}_layer")
    }
}

#[derive(Debug)]
pub(super) struct ClosingTransactionParams {
    pub(super) journal_id: JournalId,
    pub(super) description: String,
    pub(super) effective: chrono::NaiveDate,
    pub(super) closing_entries: Vec<ClosingTxEntry>,
}

impl From<ClosingTransactionParams> for Params {
    fn from(input_params: ClosingTransactionParams) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", input_params.journal_id);
        params.insert("description", input_params.description);
        params.insert("effective", input_params.effective);

        for (n, entry) in input_params.closing_entries.into_iter().enumerate() {
            EntryParams::from(entry).populate_params(&mut params, n);
        }

        params
    }
}

impl ClosingTransactionParams {
    pub(super) fn new(
        journal_id: JournalId,
        description: String,
        effective: NaiveDate,
        closing_entries: Vec<ClosingTxEntry>,
    ) -> ClosingTransactionParams {
        Self {
            journal_id,
            description,
            effective,
            closing_entries,
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
}

pub(super) struct ClosingTransactionTemplate {
    pub period_designation: String,
    pub n_entries: usize,
}

impl ClosingTransactionTemplate {
    pub(super) fn code(&self) -> String {
        format!("CLOSING_TRANSACTION_{}", self.period_designation)
    }

    pub(super) async fn init(
        op: &mut LedgerOperation<'_>,
        ledger: &CalaLedger,
        n_entries: usize,
        period_designation: String,
    ) -> Result<Self, TxTemplateError> {
        let res = Self {
            period_designation,
            n_entries,
        };
        res.find_or_create_template_in_op(op, ledger).await?;
        Ok(res)
    }

    async fn find_or_create_template_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        ledger: &CalaLedger,
    ) -> Result<(), TxTemplateError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .description("params.description")
            .effective("params.effective")
            .build()
            .expect("Couldn't build TxInput for ClosingTransactionTemplate");

        let params = ClosingTransactionParams::defs(self.n_entries);
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(self.code())
            .transaction(tx_input)
            .entries(self.entries())
            .params(params)
            .description(format!(
                "Template to execute a closing transaction with {} entries.",
                self.n_entries
            ))
            .build()
            .expect("Couldn't build template for ClosingTransactionTemplate");
        match ledger.tx_templates().create_in_op(op, template).await {
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
                        "'CLOSING_TRANSACTION_{}_ENTRY_{}'",
                        self.period_designation, i
                    ))
                    .account_id(format!("params.{}", EntryParams::account_id_param_name(i)))
                    .units(format!("params.{}", EntryParams::amount_param_name(i)))
                    .currency(format!("params.{}", EntryParams::currency_param_name(i)))
                    .layer(format!("params.{}", EntryParams::layer_param_name(i)))
                    .direction(format!("params.{}", EntryParams::direction_param_name(i)))
                    .build()
                    .expect("Couldn't build entry for ClosingTransactionTemplate"),
            );
        }
        entries
    }
}
