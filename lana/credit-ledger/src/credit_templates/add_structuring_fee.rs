use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use core_credit::{CalaAccountId, *};

pub const ADD_STRUCTURING_FEE_CODE: &str = "ADD_STRUCTURING_FEE";

#[derive(Debug)]
pub struct AddStructuringFeeParams<S: std::fmt::Display> {
    pub journal_id: JournalId,
    pub facility_fee_income_account: CalaAccountId,
    pub debit_account_id: CalaAccountId,
    pub structuring_fee_amount: Decimal,
    pub currency: Currency,
    pub external_id: String,
    pub effective: chrono::NaiveDate,
    pub initiated_by: S,
}

impl<S: std::fmt::Display> AddStructuringFeeParams<S> {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("facility_fee_income_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("debit_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("structuring_fee_amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("currency")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("external_id")
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

impl<S: std::fmt::Display> From<AddStructuringFeeParams<S>> for Params {
    fn from(
        AddStructuringFeeParams {
            journal_id,
            facility_fee_income_account,
            debit_account_id,
            structuring_fee_amount,
            currency,
            external_id,
            effective,
            initiated_by,
        }: AddStructuringFeeParams<S>,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("facility_fee_income_account", facility_fee_income_account);
        params.insert("debit_account_id", debit_account_id);
        params.insert("structuring_fee_amount", structuring_fee_amount);
        params.insert("currency", currency);
        params.insert("external_id", external_id);
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

pub struct AddStructuringFee;

impl AddStructuringFee {
    #[record_error_severity]
    #[instrument(name = "ledger.add_structuring_fee.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .metadata("params.meta")
            .description("'Add structuring fee'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            // Upfront fee collection (net funding): borrower pays fee from deposit, income recognized
            NewTxTemplateEntry::builder()
                .account_id("params.debit_account_id")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'ADD_STRUCTURING_FEE_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.facility_fee_income_account")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'ADD_STRUCTURING_FEE_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
        ];
        let params = AddStructuringFeeParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(ADD_STRUCTURING_FEE_CODE)
            .transaction(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build template");

        match ledger.tx_templates().create(template).await {
            Err(TxTemplateError::DuplicateCode) => Ok(()),
            Err(e) => Err(CreditLedgerError::from_ledger(e)),
            Ok(_) => Ok(()),
        }
    }
}
