use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};

use crate::credit_facility::ledger::error::*;

pub const CREDIT_FACILITY_INCUR_INTEREST_CODE: &str = "CREDIT_FACILITY_INCUR_INTEREST";

#[derive(Debug)]
pub struct CreditFacilityIncurInterestParams {
    pub journal_id: JournalId,
    pub credit_facility_interest_receivable_account: AccountId,
    pub credit_facility_interest_income_account: AccountId,
    pub interest_amount: Decimal,
    pub external_id: String,
    pub effective: chrono::NaiveDate,
}

impl CreditFacilityIncurInterestParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_facility_interest_receivable_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_facility_interest_income_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("interest_amount")
                .r#type(ParamDataType::Decimal)
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

impl From<CreditFacilityIncurInterestParams> for Params {
    fn from(
        CreditFacilityIncurInterestParams {
            journal_id,
            credit_facility_interest_receivable_account,
            credit_facility_interest_income_account,
            interest_amount,
            external_id,
            effective,
        }: CreditFacilityIncurInterestParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert(
            "credit_facility_interest_receivable_account",
            credit_facility_interest_receivable_account,
        );
        params.insert(
            "credit_facility_interest_income_account",
            credit_facility_interest_income_account,
        );
        params.insert("interest_amount", interest_amount);
        params.insert("external_id", external_id);
        params.insert("effective", effective);
        params
    }
}

pub struct CreditFacilityIncurInterest;

impl CreditFacilityIncurInterest {
    #[instrument(name = "ledger.credit_facility_incur_interest.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .description("'Incur interest in accrual period for credit facility'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_interest_receivable_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'INCUR_INTEREST_DR'")
                .direction("DEBIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_interest_income_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'INCUR_INTEREST_CR'")
                .direction("CREDIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = CreditFacilityIncurInterestParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(CREDIT_FACILITY_INCUR_INTEREST_CODE)
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
