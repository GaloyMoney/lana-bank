use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const MOVE_TO_LIQUIDATION_OBLIGATION_CODE: &str = "MOVE_TO_LIQUIDATION_OBLIGATION";

#[derive(Debug)]
pub struct MoveToLiquidationObligationParams {
    pub journal_id: JournalId,
    pub amount: Decimal,
    pub receivable_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

impl MoveToLiquidationObligationParams {
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
                .name("effective")
                .r#type(ParamDataType::Date)
                .build()
                .unwrap(),
        ]
    }
}
impl From<MoveToLiquidationObligationParams> for Params {
    fn from(
        MoveToLiquidationObligationParams {
            journal_id,
            amount,
            receivable_account_id,
            effective,
        }: MoveToLiquidationObligationParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("amount", amount);
        params.insert("receivable_account_id", receivable_account_id);
        params.insert("effective", effective);

        params
    }
}

pub struct MoveToLiquidationObligation;

impl MoveToLiquidationObligation {
    #[instrument(name = "ledger.move_to_liquidation_obligation.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .description("'Move an obligation into liquidation'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'MOVE_TO_LIQUIDATION_OBLIGATION_CR'")
                .currency("'USD'")
                .account_id("params.receivable_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'MOVE_TO_LIQUIDATION_OBLIGATION_DR'")
                .currency("'USD'")
                .account_id("params.receivable_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = MoveToLiquidationObligationParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(MOVE_TO_LIQUIDATION_OBLIGATION_CODE)
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
