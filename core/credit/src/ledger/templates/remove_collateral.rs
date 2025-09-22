use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{
    ledger::error::*,
    primitives::{COLLATERAL_TRANSACTION_ENTITY_TYPE, CalaAccountId},
};

pub const REMOVE_COLLATERAL_CODE: &str = "REMOVE_COLLATERAL";

#[derive(Debug)]
pub struct RemoveCollateralParams {
    pub entity_id: uuid::Uuid,
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub collateral_account_id: CalaAccountId,
    pub bank_collateral_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

impl RemoveCollateralParams {
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
                .name("collateral_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("bank_collateral_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("effective")
                .r#type(ParamDataType::Date)
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

impl From<RemoveCollateralParams> for Params {
    fn from(
        RemoveCollateralParams {
            entity_id,
            journal_id,
            currency,
            amount,
            collateral_account_id,
            bank_collateral_account_id,
            effective,
        }: RemoveCollateralParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("collateral_account_id", collateral_account_id);
        params.insert("bank_collateral_account_id", bank_collateral_account_id);
        params.insert("effective", effective);
        let entity_ref =
            core_accounting::EntityRef::new(COLLATERAL_TRANSACTION_ENTITY_TYPE, entity_id);
        params.insert("meta", serde_json::json!({"entity_ref":entity_ref}));
        params
    }
}

pub struct RemoveCollateral;

impl RemoveCollateral {
    #[instrument(name = "ledger.add_collateral.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Record a deposit'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'REMOVE_COLLATERAL_DR'")
                .currency("params.currency")
                .account_id("params.collateral_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'REMOVE_COLLATERAL_CR'")
                .currency("params.currency")
                .account_id("params.bank_collateral_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RemoveCollateralParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(REMOVE_COLLATERAL_CODE)
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
