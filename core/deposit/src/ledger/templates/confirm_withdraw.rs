use rust_decimal::Decimal;

use tracing::instrument;

use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};

use crate::ledger::error::*;

pub const CONFIRM_WITHDRAW_CODE: &str = "CONFIRM_WITHDRAW_CODE";

#[derive(Debug)]
pub struct ConfirmWithdrawParams {
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub deposit_omnibus_account_id: AccountId,
    pub credit_account_id: AccountId,
    pub correlation_id: String,
    pub external_id: String,
}

impl ConfirmWithdrawParams {
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
            NewParamDefinition::builder()
                .name("correlation_id")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("external_id")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
        ]
    }
}

impl From<ConfirmWithdrawParams> for Params {
    fn from(
        ConfirmWithdrawParams {
            journal_id,
            currency,
            amount,
            deposit_omnibus_account_id,
            correlation_id,
            external_id,
            credit_account_id,
        }: ConfirmWithdrawParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("deposit_omnibus_account_id", deposit_omnibus_account_id);
        params.insert("credit_account_id", credit_account_id);
        params.insert("correlation_id", correlation_id);
        params.insert("external_id", external_id);
        params.insert("effective", chrono::Utc::now().date_naive());

        params
    }
}

pub struct ConfirmWithdraw;

impl ConfirmWithdraw {
    #[instrument(name = "ledger.confirm_withdraw.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), DepositLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .description("'Confirm a withdraw'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            // check in graphql/cancel-withdraw the entry type
            NewTxTemplateEntry::builder()
                .entry_type("'CONFIRM_WITHDRAW_PENDING_CR'")
                .currency("params.currency")
                .account_id("params.deposit_omnibus_account_id")
                .direction("CREDIT")
                .layer("PENDING")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'CONFIRM_WITHDRAW_PENDING_DR'")
                .currency("params.currency")
                .account_id("params.credit_account_id")
                .direction("DEBIT")
                .layer("PENDING")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = ConfirmWithdrawParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(CONFIRM_WITHDRAW_CODE)
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
