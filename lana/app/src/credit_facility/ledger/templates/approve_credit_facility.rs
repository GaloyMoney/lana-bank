use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};

use crate::credit_facility::ledger::error::*;

pub const APPROVE_CREDIT_FACILITY_CODE: &str = "APPROVE_CREDIT_FACILITY";

#[derive(Debug)]
pub struct ApproveCreditFacilityParams {
    pub journal_id: JournalId,
    pub credit_omnibus_account: AccountId,
    pub credit_facility_account: AccountId,
    pub facility_disbursed_receivable_account: AccountId,
    pub facility_fee_income_account: AccountId,
    pub checking_account: AccountId,
    pub facility_amount: Decimal,
    pub structuring_fee_amount: Decimal,
    pub currency: Currency,
    pub external_id: String,
}

impl ApproveCreditFacilityParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_omnibus_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_facility_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("facility_disbursed_receivable_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("facility_fee_income_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("checking_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("facility_amount")
                .r#type(ParamDataType::Decimal)
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

impl From<ApproveCreditFacilityParams> for Params {
    fn from(
        ApproveCreditFacilityParams {
            journal_id,
            credit_omnibus_account,
            credit_facility_account,
            facility_disbursed_receivable_account,
            facility_fee_income_account,
            checking_account,
            facility_amount,
            structuring_fee_amount,
            currency,
            external_id,
        }: ApproveCreditFacilityParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("credit_facility_account", credit_facility_account);
        params.insert("credit_omnibus_account", credit_omnibus_account);
        params.insert(
            "facility_disbursed_receivable_account",
            facility_disbursed_receivable_account,
        );
        params.insert("facility_fee_income_account", facility_fee_income_account);
        params.insert("checking_account", checking_account);
        params.insert("facility_amount", facility_amount);
        params.insert("structuring_fee_amount", structuring_fee_amount);
        params.insert("currency", currency);
        params.insert("external_id", external_id);
        params.insert("effective", chrono::Utc::now().date_naive());
        params
    }
}

pub struct ApproveCreditFacility;

impl ApproveCreditFacility {
    #[instrument(name = "ledger.approve_credit_facility.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .description("'Approve credit facility'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .account_id("params.credit_omnibus_account")
                .units("params.facility_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_account")
                .units("params.facility_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_account")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_DISBURSEMENT_DRAWDOWN_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_omnibus_account")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_DISBURSEMENT_DRAWDOWN_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.facility_disbursed_receivable_account")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_DISBURSEMENT_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.checking_account")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_DISBURSEMENT_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.checking_account")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_STRUCTURING_FEE_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.facility_fee_income_account")
                .units("params.structuring_fee_amount")
                .currency("params.currency")
                .entry_type("'APPROVE_CREDIT_FACILITY_STRUCTURING_FEE_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
        ];
        let params = ApproveCreditFacilityParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(APPROVE_CREDIT_FACILITY_CODE)
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
