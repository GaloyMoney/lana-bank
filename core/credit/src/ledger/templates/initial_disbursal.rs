use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{
    ledger::error::*,
    primitives::{CalaAccountId, DISBURSAL_TRANSACTION_ENTITY_TYPE},
};

pub const INITIAL_DISBURSAL_CODE: &str = "INITIAL_DISBURSAL";

#[derive(Debug)]
pub struct InitialDisbursalParams {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub credit_omnibus_account: CalaAccountId,
    pub credit_facility_account: CalaAccountId,
    pub facility_disbursed_receivable_account: CalaAccountId,
    pub debit_account_id: CalaAccountId,
    pub disbursed_amount: Decimal,
    pub currency: Currency,
    pub external_id: String,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
}

impl InitialDisbursalParams {
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
                .name("debit_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("disbursed_amount")
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

impl From<InitialDisbursalParams> for Params {
    fn from(
        InitialDisbursalParams {
            entity_id,
            journal_id,
            credit_omnibus_account,
            credit_facility_account,
            facility_disbursed_receivable_account,
            debit_account_id,
            disbursed_amount,
            currency,
            external_id,
            initiated_by,
        }: InitialDisbursalParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("credit_omnibus_account", credit_omnibus_account);
        params.insert("credit_facility_account", credit_facility_account);
        params.insert(
            "facility_disbursed_receivable_account",
            facility_disbursed_receivable_account,
        );
        params.insert("debit_account_id", debit_account_id);
        params.insert("disbursed_amount", disbursed_amount);
        params.insert("currency", currency);
        params.insert("external_id", external_id);
        params.insert("effective", crate::time::now().date_naive());
        let entity_ref =
            core_accounting::EntityRef::new(DISBURSAL_TRANSACTION_ENTITY_TYPE, entity_id);
        params.insert(
            "meta",
            serde_json::json!({
                "entity_ref": entity_ref,
                "initiated_by": initiated_by,
            }),
        );
        params
    }
}

pub struct InitialDisbursal;

impl InitialDisbursal {
    #[record_error_severity]
    #[instrument(name = "ledger.initial_disbursal.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .metadata("params.meta")
            .description("'Initial disbursal'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_account")
                .units("params.disbursed_amount")
                .currency("params.currency")
                .entry_type("'SINGLE_DISBURSAL_DRAWDOWN_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_omnibus_account")
                .units("params.disbursed_amount")
                .currency("params.currency")
                .entry_type("'SINGLE_DISBURSAL_DRAWDOWN_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.facility_disbursed_receivable_account")
                .units("params.disbursed_amount")
                .currency("params.currency")
                .entry_type("'SINGLE_DISBURSAL_RECEIVABLE_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.debit_account_id")
                .units("params.disbursed_amount")
                .currency("params.currency")
                .entry_type("'SINGLE_DISBURSAL_RECEIVABLE_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
        ];
        let params = InitialDisbursalParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(INITIAL_DISBURSAL_CODE)
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
