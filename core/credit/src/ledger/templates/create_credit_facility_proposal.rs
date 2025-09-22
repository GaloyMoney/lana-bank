use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{
    ledger::error::*,
    primitives::{CREDIT_FACILITY_PROPOSAL_TRANSACTION_ENTITY_TYPE, CalaAccountId},
};

pub const CREATE_CREDIT_FACILITY_PROPOSAL_CODE: &str = "CREATE_CREDIT_FACILITY_PROPOSAL";

#[derive(Debug)]
pub struct CreateCreditFacilityProposalParams {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub credit_omnibus_account: CalaAccountId,
    pub credit_facility_account: CalaAccountId,
    pub facility_amount: Decimal,
    pub currency: Currency,
    pub external_id: String,
}

impl CreateCreditFacilityProposalParams {
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
            NewParamDefinition::builder()
                .name("meta")
                .r#type(ParamDataType::Json)
                .build()
                .unwrap(),
        ]
    }
}

impl From<CreateCreditFacilityProposalParams> for Params {
    fn from(
        CreateCreditFacilityProposalParams {
            entity_id,
            journal_id,
            credit_omnibus_account,
            credit_facility_account,
            facility_amount,
            currency,
            external_id,
        }: CreateCreditFacilityProposalParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("credit_facility_account", credit_facility_account);
        params.insert("credit_omnibus_account", credit_omnibus_account);
        params.insert("facility_amount", facility_amount);
        params.insert("currency", currency);
        params.insert("external_id", external_id);
        params.insert("effective", crate::time::now().date_naive());
        let entity_ref = core_accounting::EntityRef::new(
            CREDIT_FACILITY_PROPOSAL_TRANSACTION_ENTITY_TYPE,
            entity_id,
        );
        params.insert("meta", serde_json::json!({"entity_ref":entity_ref}));
        params
    }
}

pub struct CreateCreditFacilityProposal;

impl CreateCreditFacilityProposal {
    #[instrument(name = "ledger.create_credit_facility.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .metadata("params.meta")
            .description("'Create credit facility'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .account_id("params.credit_omnibus_account")
                .units("params.facility_amount")
                .currency("params.currency")
                .entry_type("'CREATE_CREDIT_FACILITY_PROPOSAL_DR'")
                .direction("DEBIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_account")
                .units("params.facility_amount")
                .currency("params.currency")
                .entry_type("'CREATE_CREDIT_FACILITY_PROPOSAL_CR'")
                .direction("CREDIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
        ];
        let params = CreateCreditFacilityProposalParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(CREATE_CREDIT_FACILITY_PROPOSAL_CODE)
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
