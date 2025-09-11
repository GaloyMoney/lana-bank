use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const CREATE_CREDIT_FACILITY_CODE: &str = "CREATE_CREDIT_FACILITY";

#[derive(Debug)]
pub struct CreateCreditFacilityParams {
    pub journal_id: JournalId,
    pub credit_omnibus_account: CalaAccountId,
    pub credit_facility_account: CalaAccountId,
    pub facility_amount: Decimal,
    pub currency: Currency,
    pub external_id: String,
}

impl CreateCreditFacilityParams {
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
                .name("facility_amount")
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

impl From<CreateCreditFacilityParams> for Params {
    fn from(
        CreateCreditFacilityParams {
            journal_id,
            credit_omnibus_account,
            credit_facility_account,
            facility_amount,
            currency,
            external_id,
        }: CreateCreditFacilityParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("credit_facility_account", credit_facility_account);
        params.insert("credit_omnibus_account", credit_omnibus_account);
        params.insert("facility_amount", facility_amount);
        params.insert("currency", currency);
        params.insert("external_id", external_id);
        params.insert("effective", crate::time::now().date_naive());
        params
    }
}

pub struct CreateCreditFacility;

impl CreateCreditFacility {
    #[instrument(name = "ledger.create_credit_facility.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .description("'Create credit facility'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .account_id("params.credit_omnibus_account")
                .units("params.facility_amount")
                .currency("params.currency")
                .entry_type("'CREATE_CREDIT_FACILITY_DR'")
                .direction("DEBIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_account")
                .units("params.facility_amount")
                .currency("params.currency")
                .entry_type("'CREATE_CREDIT_FACILITY_CR'")
                .direction("CREDIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
        ];
        let params = CreateCreditFacilityParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(CREATE_CREDIT_FACILITY_CODE)
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
