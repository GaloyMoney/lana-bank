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

pub const CANCEL_DISBURSAL_CODE: &str = "CANCEL_DISBURSAL";

#[derive(Debug)]
pub struct CancelDisbursalParams<S: std::fmt::Display> {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub facility_uncovered_outstanding_account: CalaAccountId,
    pub credit_facility_account: CalaAccountId,
    pub disbursed_amount: Decimal,
    pub effective: chrono::NaiveDate,
    pub initiated_by: S,
}

impl<S: std::fmt::Display> CancelDisbursalParams<S> {
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

impl<S: std::fmt::Display> From<CancelDisbursalParams<S>> for Params {
    fn from(
        CancelDisbursalParams {
            entity_id,
            journal_id,
            facility_uncovered_outstanding_account,
            credit_facility_account,
            disbursed_amount,
            effective,
            initiated_by,
        }: CancelDisbursalParams<S>,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert(
            "facility_uncovered_outstanding_account",
            facility_uncovered_outstanding_account,
        );
        params.insert("credit_facility_account", credit_facility_account);
        params.insert("disbursed_amount", disbursed_amount);
        params.insert("effective", effective);
        let entity_ref = core_accounting_contracts::EntityRef::new(
            DISBURSAL_TRANSACTION_ENTITY_TYPE,
            entity_id,
        );
        params.insert(
            "meta",
            serde_json::json!({
                "entity_ref": entity_ref,
                "initiated_by": initiated_by.to_string(),
            }),
        );
        params
    }
}

pub struct CancelDisbursal;

impl CancelDisbursal {
    #[record_error_severity]
    #[instrument(name = "ledger.cancel_disbursal.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Cancel a disbursal'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            // Reverse pending entries
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_DISBURSAL_DRAWDOWN_PENDING_DR'")
                .currency("'USD'")
                .account_id("params.credit_facility_account")
                .direction("DEBIT")
                .layer("PENDING")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_DISBURSAL_DRAWDOWN_PENDING_CR'")
                .currency("'USD'")
                .account_id("params.facility_uncovered_outstanding_account")
                .direction("CREDIT")
                .layer("PENDING")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
            // Reverse settled entries
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_DISBURSAL_DRAWDOWN_SETTLED_DR'")
                .currency("'USD'")
                .account_id("params.facility_uncovered_outstanding_account")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_DISBURSAL_DRAWDOWN_SETTLED_CR'")
                .currency("'USD'")
                .account_id("params.credit_facility_account")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = CancelDisbursalParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(CANCEL_DISBURSAL_CODE)
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
