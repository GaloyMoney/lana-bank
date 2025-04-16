use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const RECORD_PAYMENT_ALLOCATION_CODE: &str = "RECORD_PAYMENT_ALLOCATION";

#[derive(Debug)]
pub struct RecordPaymentAllocationParams {
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub account_to_be_debited_id: CalaAccountId,
    pub receivable_account_id: CalaAccountId,
    pub tx_ref: String,
}

impl RecordPaymentAllocationParams {
    pub fn defs() -> Vec<NewParamDefinition> {
        vec![
            NewParamDefinition::builder()
                .name("external_id")
                .r#type(ParamDataType::String)
                .build()
                .unwrap(),
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
                .name("account_to_be_debited_id")
                .r#type(ParamDataType::Uuid)
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
impl From<RecordPaymentAllocationParams> for Params {
    fn from(
        RecordPaymentAllocationParams {
            journal_id,
            currency,
            amount,
            account_to_be_debited_id,
            receivable_account_id,
            tx_ref,
        }: RecordPaymentAllocationParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("external_id", tx_ref);
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("account_to_be_debited_id", account_to_be_debited_id);
        params.insert("receivable_account_id", receivable_account_id);
        params.insert("effective", chrono::Utc::now().date_naive());

        params
    }
}

pub struct RecordPaymentAllocation;

impl RecordPaymentAllocation {
    #[instrument(name = "ledger.record_payment.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .description("'Record a deposit'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_PAYMENT_ALLOCATION_DR'")
                .currency("params.currency")
                .account_id("params.account_to_be_debited_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.principal_amount + params.interest_amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_PAYMENT_ALLOCATION_CR'")
                .currency("params.currency")
                .account_id("params.receivable_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RecordPaymentAllocationParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RECORD_PAYMENT_ALLOCATION_CODE)
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
