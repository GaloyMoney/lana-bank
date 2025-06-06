use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::ledger::error::*;

pub const CANCEL_WITHDRAW_CODE: &str = "CANCEL_WITHDRAW";

#[derive(Debug)]
pub struct CancelWithdrawParams {
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub deposit_omnibus_account_id: AccountId,
    pub credit_account_id: AccountId,
}

impl CancelWithdrawParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("journal_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("currency")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("deposit_omnibus_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("credit_account_id")
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

impl From<CancelWithdrawParams> for Params {
    fn from(
        CancelWithdrawParams {
            journal_id,
            currency,
            amount,
            deposit_omnibus_account_id,
            credit_account_id,
        }: CancelWithdrawParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("deposit_omnibus_account_id", deposit_omnibus_account_id);
        params.insert("credit_account_id", credit_account_id);
        params.insert("effective", crate::time::now().date_naive());

        params
    }
}

pub struct CancelWithdraw;

impl CancelWithdraw {
    #[instrument(name = "ledger.cancel_withdraw.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), DepositLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .description("'Cancel a Withdraw'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_WITHDRAW_PENDING_CR'")
                .currency("params.currency")
                .account_id("params.deposit_omnibus_account_id")
                .direction("CREDIT")
                .layer("PENDING")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_WITHDRAW_PENDING_DR'")
                .currency("params.currency")
                .account_id("params.credit_account_id")
                .direction("DEBIT")
                .layer("PENDING")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_WITHDRAW_SETTLED_DR'")
                .currency("params.currency")
                .account_id("params.deposit_omnibus_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'CANCEL_WITHDRAW_SETTLED_CR'")
                .currency("params.currency")
                .account_id("params.credit_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = CancelWithdrawParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(CANCEL_WITHDRAW_CODE)
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
