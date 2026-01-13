use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const CREDIT_FACILITY_POST_ACCRUED_INTEREST_CODE: &str =
    "CREDIT_FACILITY_POST_ACCRUED_INTEREST";

#[derive(Debug)]
pub struct CreditFacilityPostAccruedInterestParams {
    pub journal_id: JournalId,
    pub credit_facility_interest_receivable_account: CalaAccountId,
    pub credit_facility_interest_income_account: CalaAccountId,
    pub interest_added_to_obligations_omnibus_account: CalaAccountId,
    pub credit_facility_uncovered_outstanding_account: CalaAccountId,
    pub interest_amount: Decimal,
    pub external_id: String,
    pub effective: chrono::NaiveDate,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
}

impl CreditFacilityPostAccruedInterestParams {
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
                .name("interest_added_to_obligations_omnibus_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_facility_uncovered_outstanding_account")
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
            NewParamDefinition::builder()
                .name("meta")
                .r#type(ParamDataType::Json)
                .build()
                .unwrap(),
        ]
    }
}

impl From<CreditFacilityPostAccruedInterestParams> for Params {
    fn from(
        CreditFacilityPostAccruedInterestParams {
            journal_id,
            credit_facility_interest_receivable_account,
            credit_facility_interest_income_account,
            interest_added_to_obligations_omnibus_account,
            credit_facility_uncovered_outstanding_account,
            interest_amount,
            external_id,
            effective,
            initiated_by,
        }: CreditFacilityPostAccruedInterestParams,
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
        params.insert(
            "interest_added_to_obligations_omnibus_account",
            interest_added_to_obligations_omnibus_account,
        );
        params.insert(
            "credit_facility_uncovered_outstanding_account",
            credit_facility_uncovered_outstanding_account,
        );
        params.insert("interest_amount", interest_amount);
        params.insert("external_id", external_id);
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

pub struct CreditFacilityPostAccruedInterest;

impl CreditFacilityPostAccruedInterest {
    #[record_error_severity]
    #[instrument(name = "ledger.credit_facility_post_accrued_interest.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .metadata("params.meta")
            .description("'Post accrued interest from accrual cycle for credit facility'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            // Reverse pending interest accrual entries
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_interest_income_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'POST_ACCRUED_INTEREST_PENDING_DR'")
                .direction("DEBIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_interest_receivable_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'POST_ACCRUED_INTEREST_PENDING_CR'")
                .direction("CREDIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
            // SETTLED LAYER interest entries (not yet due)
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_interest_receivable_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'POST_ACCRUED_INTEREST_SETTLED_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_interest_income_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'POST_ACCRUED_INTEREST_SETTLED_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.interest_added_to_obligations_omnibus_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'RECORD_INTEREST_UNCOVERED_OBLIGATION_SETTLED_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_uncovered_outstanding_account")
                .units("params.interest_amount")
                .currency("'USD'")
                .entry_type("'RECORD_INTEREST_UNCOVERED_OBLIGATION_SETTLED_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = CreditFacilityPostAccruedInterestParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(CREDIT_FACILITY_POST_ACCRUED_INTEREST_CODE)
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
