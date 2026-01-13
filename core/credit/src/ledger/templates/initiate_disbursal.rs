use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};
use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    ledger::error::*,
    primitives::{CalaAccountId, DISBURSAL_TRANSACTION_ENTITY_TYPE},
};

pub const INITIATE_DISBURSAL_CODE: &str = "INITIATE_CREDIT_FACILITY_DISBURSAL";

#[derive(Debug)]
pub struct InitiateDisbursalParams {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub facility_uncovered_outstanding_account: CalaAccountId,
    pub credit_facility_account: CalaAccountId,
    pub disbursed_amount: Decimal,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
}

impl InitiateDisbursalParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("facility_uncovered_outstanding_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_facility_account")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("disbursed_amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("effective")
                .r#type(ParamDataType::Date)
                .description("Effective date for transaction.")
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

impl From<InitiateDisbursalParams> for Params {
    fn from(
        InitiateDisbursalParams {
            entity_id,
            journal_id,
            credit_facility_account,
            disbursed_amount,
            facility_uncovered_outstanding_account,
            initiated_by,
        }: InitiateDisbursalParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert(
            "facility_uncovered_outstanding_account",
            facility_uncovered_outstanding_account,
        );
        params.insert("credit_facility_account", credit_facility_account);
        params.insert("disbursed_amount", disbursed_amount);
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

pub struct InitiateDisbursal;

impl InitiateDisbursal {
    #[record_error_severity]
    #[instrument(name = "ledger.initiate_disbursal.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Initiate credit facility disbursal.'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            // SETTLED layer entries
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_account")
                .units("params.disbursed_amount")
                .currency("'USD'")
                .entry_type("'INITIATE_DISBURSAL_DRAWDOWN_SETTLED_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.facility_uncovered_outstanding_account")
                .units("params.disbursed_amount")
                .currency("'USD'")
                .entry_type("'INITIATE_DISBURSAL_DRAWDOWN_SETTLED_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            // PENDING layer entries
            NewTxTemplateEntry::builder()
                .account_id("params.facility_uncovered_outstanding_account")
                .units("params.disbursed_amount")
                .currency("'USD'")
                .entry_type("'INITIATE_DISBURSAL_DRAWDOWN_PENDING_DR'")
                .direction("DEBIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.credit_facility_account")
                .units("params.disbursed_amount")
                .currency("'USD'")
                .entry_type("'INITIATE_DISBURSAL_DRAWDOWN_PENDING_CR'")
                .direction("CREDIT")
                .layer("PENDING")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = InitiateDisbursalParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(INITIATE_DISBURSAL_CODE)
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
