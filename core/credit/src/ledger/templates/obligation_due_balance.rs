use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const RECORD_OBLIGATION_DUE_BALANCE_CODE: &str = "RECORD_OBLIGATION_DUE_BALANCE";

#[derive(Debug)]
pub struct RecordObligationDueBalanceParams {
    pub journal_id: JournalId,
    pub amount: Decimal,
    pub receivable_account_id: CalaAccountId,
    pub account_to_be_credited_id: CalaAccountId,
}

impl RecordObligationDueBalanceParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("receivable_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("account_to_be_credited_id")
                .r#type(ParamDataType::Uuid)
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
impl From<RecordObligationDueBalanceParams> for Params {
    fn from(
        RecordObligationDueBalanceParams {
            journal_id,
            amount,
            receivable_account_id,
            account_to_be_credited_id,
        }: RecordObligationDueBalanceParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("amount", amount);
        params.insert("receivable_account_id", receivable_account_id);
        params.insert("account_to_be_credited_id", account_to_be_credited_id);
        params.insert("effective", chrono::Utc::now().date_naive());

        params
    }
}

pub struct RecordObligationDueBalance;

impl RecordObligationDueBalance {
    #[instrument(name = "ledger.record_obligation_overdue_balance.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .description("'Record an overdue obligation balance'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'REVERT_OBLIGATION_PENDING_BALANCE_DR'")
                .currency("'USD'")
                .account_id("params.account_to_be_credited_id")
                .direction("DEBIT")
                .layer("PENDING")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'REVERT_OBLIGATION_PENDING_BALANCE_CR'")
                .currency("'USD'")
                .account_id("params.receivable_account_id")
                .direction("CREDIT")
                .layer("PENDING")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_OBLIGATION_DUE_BALANCE_DR'")
                .currency("'USD'")
                .account_id("params.receivable_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_OBLIGATION_DUE_BALANCE_CR'")
                .currency("'USD'")
                .account_id("params.account_to_be_credited_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RecordObligationDueBalanceParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RECORD_OBLIGATION_DUE_BALANCE_CODE)
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
