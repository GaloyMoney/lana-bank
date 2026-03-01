use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{
    ledger::error::*,
    primitives::{CalaAccountId, DEPOSIT_TRANSACTION_ENTITY_TYPE},
};

pub const REVERT_DEPOSIT_CODE: &str = "REVERT_DEPOSIT";

#[derive(Debug)]
pub struct RevertDepositParams<S: std::fmt::Display> {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub deposit_omnibus_account_id: CalaAccountId,
    pub credit_account_id: CalaAccountId,
    pub amount: Decimal,
    pub currency: Currency,
    pub correlation_id: String,
    pub external_id: String,
    pub initiated_by: S,
    pub effective_date: chrono::NaiveDate,
}

impl<S: std::fmt::Display> RevertDepositParams<S> {
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

impl<S: std::fmt::Display> From<RevertDepositParams<S>> for Params {
    fn from(
        RevertDepositParams {
            entity_id,
            journal_id,
            deposit_omnibus_account_id,
            credit_account_id,
            amount,
            currency,
            correlation_id,
            external_id,
            initiated_by,
            effective_date,
        }: RevertDepositParams<S>,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("deposit_omnibus_account_id", deposit_omnibus_account_id);
        params.insert("credit_account_id", credit_account_id);
        params.insert("amount", amount);
        params.insert("currency", currency);
        params.insert("correlation_id", correlation_id);
        params.insert("external_id", external_id);

        params.insert("effective", effective_date);
        let entity_ref =
            chart_primitives::EntityRef::new(DEPOSIT_TRANSACTION_ENTITY_TYPE, entity_id);
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

pub struct RevertDeposit;

impl RevertDeposit {
    #[record_error_severity]
    #[instrument(name = "ledger.revert_deposit.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), DepositLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Revert a deposit'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'REVERT_DEPOSIT_CR'")
                .currency("params.currency")
                .account_id("params.deposit_omnibus_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'REVERT_DEPOSIT_DR'")
                .currency("params.currency")
                .account_id("params.credit_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RevertDepositParams::<String>::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(REVERT_DEPOSIT_CODE)
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
