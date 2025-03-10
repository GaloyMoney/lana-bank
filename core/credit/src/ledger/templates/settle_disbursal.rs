use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};
use rust_decimal::Decimal;
use tracing::instrument;

use crate::ledger::error::*;

pub const SETTLE_DISBURSAL_CODE: &str = "SETTLE_DISBURSAL_CODE";

#[derive(Debug)]
pub struct SettleDisbursalParams {
    pub journal_id: JournalId,
    pub credit_omnibus_account: AccountId,
    pub credit_facility_account: AccountId,
    pub facility_disbursed_receivable_account: AccountId,
    pub debit_account_id: AccountId,
    pub disbursed_amount: Decimal,
    pub external_id: String,
}

impl SettleDisbursalParams {
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

impl From<SettleDisbursalParams> for Params {
    fn from(
        SettleDisbursalParams {
            journal_id,
            credit_omnibus_account,
            credit_facility_account,
            facility_disbursed_receivable_account,
            debit_account_id,
            disbursed_amount,
            external_id,
        }: SettleDisbursalParams,
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
        params.insert("external_id", external_id);
        params.insert("effective", chrono::Utc::now().date_naive());
        params
    }
}

pub struct SettleDisbursal;

impl SettleDisbursal {
    #[instrument(name = "ledger.settle_disbursal.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .description("'Settle a disbursal'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            // Reverse pending entries
            NewTxTemplateEntry::builder()
                .entry_type("'SETTLE_DISBURSAL_DRAWDOWN_PENDING_DR'")
                .currency("'USD'")
                .account_id("params.credit_facility_account")
                .direction("DEBIT")
                .layer("PENDING")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'SETTLE_DISBURSAL_DRAWDOWN_PENDING_CR'")
                .currency("'USD'")
                .account_id("params.credit_omnibus_account")
                .direction("CREDIT")
                .layer("PENDING")
                .units("params.disbursed_amount")
                .build()
                .expect("Couldn't build entry"),
            // SETTLED LAYER
            NewTxTemplateEntry::builder()
                .account_id("params.facility_disbursed_receivable_account")
                .units("params.disbursed_amount")
                .currency("'USD'")
                .entry_type("'SETTLE_DISBURSAL_SETTLED_DR'")
                .direction("DEBIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .account_id("params.debit_account_id")
                .units("params.disbursed_amount")
                .currency("'USD'")
                .entry_type("'SETTLE_DISBURSAL_SETTLED_CR'")
                .direction("CREDIT")
                .layer("SETTLED")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = SettleDisbursalParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(SETTLE_DISBURSAL_CODE)
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
