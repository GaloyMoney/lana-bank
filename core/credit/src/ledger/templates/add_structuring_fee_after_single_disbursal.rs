use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const ADD_STRUCTURING_FEE_AFTER_SINGLE_DISBURSAL_CODE: &str =
    "ADD_STRUCTURING_FEE_AFTER_SINGLE_DISBURSAL";

#[derive(Debug)]
pub struct AddStructuringFeeAfterSingleDisbursalParams {
    pub journal_id: JournalId,
    pub facility_fee_income_account: CalaAccountId,
    pub debit_account_id: CalaAccountId,
    pub structuring_fee_amount: Decimal,
    pub currency: Currency,
    pub external_id: String,
}

impl AddStructuringFeeAfterSingleDisbursalParams {
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
        ]
    }
}

impl From<AddStructuringFeeAfterSingleDisbursalParams> for Params {
    fn from(
        AddStructuringFeeAfterSingleDisbursalParams {
            journal_id,
            facility_fee_income_account,
            debit_account_id,
            structuring_fee_amount,
            currency,
            external_id,
        }: AddStructuringFeeAfterSingleDisbursalParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("facility_fee_income_account", facility_fee_income_account);
        params.insert("debit_account_id", debit_account_id);
        params.insert("structuring_fee_amount", structuring_fee_amount);
        params.insert("currency", currency);
        params.insert("external_id", external_id);
        params.insert("effective", crate::time::now().date_naive());
        params
    }
}

pub struct AddStructuringFeeAfterSingleDisbursal;

impl AddStructuringFeeAfterSingleDisbursal {
    #[instrument(
        name = "ledger.add_structuring_fee_after_single_disbursal.init",
        skip_all
    )]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .description("'Add structuring fee after single disbursal'")
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
        let params = AddStructuringFeeAfterSingleDisbursalParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(ADD_STRUCTURING_FEE_AFTER_SINGLE_DISBURSAL_CODE)
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
