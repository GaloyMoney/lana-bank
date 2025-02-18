use crate::credit_facility::ledger::error::*;
use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};
use rust_decimal::Decimal;
use tracing::instrument;

pub const CANCEL_DISBURSAL_CODE: &str = "CANCEL_DISBURSAL_CODE";

#[derive(Debug)]
pub struct CancelDisbursalParams {
    pub journal_id: JournalId,
    pub credit_omnibus_account: AccountId,
    pub credit_facility_account: AccountId,
    pub disbursed_amount: Decimal,
}

impl CancelDisbursalParams {
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
                .name("disbursed_amount")
                .r#type(ParamDataType::Decimal)
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

impl From<CancelDisbursalParams> for Params {
    fn from(
        CancelDisbursalParams {
            journal_id,
            credit_omnibus_account,
            credit_facility_account,
            disbursed_amount,
        }: CancelDisbursalParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("credit_omnibus_account", credit_omnibus_account);
        params.insert("credit_facility_account", credit_facility_account);
        params.insert("disbursed_amount", disbursed_amount);
        params.insert("effective", chrono::Utc::now().date_naive());
        params
    }
}

pub struct CancelDisbursal;

impl CancelDisbursal {
    #[instrument(name = "ledger.cancel_disbursal.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
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
                .account_id("params.credit_omnibus_account")
                .direction("CREDIT")
                .layer("PENDING")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
            // Reverse settled entries
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_DISBURSAL_DRAWDOWN_SETTLED_CR'")
                .currency("'USD'")
                .account_id("params.credit_facility_account")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_DISBURSAL_DRAWDOWN_SETTLED_DR'")
                .currency("'USD'")
                .account_id("params.credit_omnibus_account")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = CancelDisbursalParams::defs();
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
