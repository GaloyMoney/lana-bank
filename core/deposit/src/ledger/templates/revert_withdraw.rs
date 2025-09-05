use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{
    ledger::error::*,
    primitives::{CalaAccountId, WITHDRAWAL_TRANSACTION_ENTITY_TYPE},
};

pub const REVERT_WITHDRAW_CODE: &str = "REVERT_WITHDRAW";

#[derive(Debug)]
pub struct RevertWithdrawParams {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub deposit_omnibus_account_id: CalaAccountId,
    pub credit_account_id: CalaAccountId,
    pub amount: Decimal,
    pub currency: Currency,
    pub correlation_id: String,
    pub external_id: String,
}

impl RevertWithdrawParams {
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
                .name("correlation_id")
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

impl From<RevertWithdrawParams> for Params {
    fn from(
        RevertWithdrawParams {
            entity_id,
            journal_id,
            deposit_omnibus_account_id,
            credit_account_id,
            amount,
            currency,
            correlation_id,
            external_id,
        }: RevertWithdrawParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("deposit_omnibus_account_id", deposit_omnibus_account_id);
        params.insert("credit_account_id", credit_account_id);
        params.insert("correlation_id", correlation_id);
        params.insert("external_id", external_id);
        params.insert("effective", crate::time::now().date_naive());
        let entity_ref =
            core_accounting::EntityRef::new(WITHDRAWAL_TRANSACTION_ENTITY_TYPE, entity_id);
        params.insert("meta", serde_json::json!({"entity_ref": entity_ref}));

        params
    }
}

pub struct RevertWithdraw;

impl RevertWithdraw {
    #[instrument(name = "ledger.revert_withdraw.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), DepositLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .description("'Revert a withdraw'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'REVERT_WITHDRAW_SETTLED_DR'")
                .currency("params.currency")
                .account_id("params.deposit_omnibus_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .unwrap(),
            NewTxTemplateEntry::builder()
                .entry_type("'REVERT_WITHDRAW_SETTLED_CR'")
                .currency("params.currency")
                .account_id("params.credit_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .unwrap(),
        ];

        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(REVERT_WITHDRAW_CODE)
            .transaction(tx_input)
            .entries(entries)
            .params(RevertWithdrawParams::defs())
            .build()
            .expect("Couldn't build template");

        match ledger.tx_templates().create(template).await {
            Err(TxTemplateError::DuplicateCode) => Ok(()),
            Err(e) => Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
}
