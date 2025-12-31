use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, Currency, JournalId, TxTemplateId,
    tx_template::{
        NewParamDefinition, NewTxTemplate, NewTxTemplateEntry, NewTxTemplateTransaction,
        ParamDataType, Params, error::TxTemplateError,
    },
};

use crate::ledger::error::DepositLedgerError;

pub const UNFREEZE_ACCOUNT_CODE: &str = "UNFREEZE_ACCOUNT";

#[derive(Debug)]
pub struct UnfreezeAccountParams {
    pub journal_id: JournalId,
    pub account_id: CalaAccountId,
    pub frozen_accounts_account_id: CalaAccountId,
    pub amount: Decimal,
    pub currency: Currency,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
}

impl UnfreezeAccountParams {
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
                .name("account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("frozen_accounts_account_id")
                .r#type(ParamDataType::Uuid)
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

impl From<UnfreezeAccountParams> for Params {
    fn from(
        UnfreezeAccountParams {
            journal_id,
            account_id,
            frozen_accounts_account_id,
            amount,
            currency,
            initiated_by,
        }: UnfreezeAccountParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("account_id", account_id);
        params.insert("frozen_accounts_account_id", frozen_accounts_account_id);
        params.insert("effective", crate::time::now().date_naive());
        params.insert(
            "meta",
            serde_json::json!({
                "initiated_by": initiated_by,
            }),
        );
        params
    }
}

pub struct UnfreezeAccount;

impl UnfreezeAccount {
    #[record_error_severity]
    #[instrument(name = "ledger.unfreeze_account.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), DepositLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .description("'Unfreeze a deposit account'")
            .effective("params.effective")
            .metadata("params.meta")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'UNFREEZE_ACCOUNT_DR'")
                .currency("params.currency")
                .account_id("params.frozen_accounts_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'UNFREEZE_ACCOUNT_CR'")
                .currency("params.currency")
                .account_id("params.account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = UnfreezeAccountParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(UNFREEZE_ACCOUNT_CODE)
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
