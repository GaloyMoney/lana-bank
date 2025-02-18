use rust_decimal::Decimal;
use tracing::instrument;

use cala_ledger::{
    tx_template::{error::TxTemplateError, Params, *},
    *,
};

use crate::credit_facility::ledger::error::*;

pub const RECORD_PAYMENT_CODE: &str = "RECORD_PAYMENT";

#[derive(Debug)]
pub struct RecordPaymentParams {
    pub journal_id: JournalId,
    pub currency: Currency,
    pub interest_amount: Decimal,
    pub principal_amount: Decimal,
    pub debit_account_id: AccountId,
    pub principal_receivable_account_id: AccountId,
    pub interest_receivable_account_id: AccountId,
    pub tx_ref: String,
}

impl RecordPaymentParams {
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
                .name("interest_amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("principal_amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("debit_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("principal_receivable_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("interest_receivable_account_id")
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
impl From<RecordPaymentParams> for Params {
    fn from(
        RecordPaymentParams {
            journal_id,
            currency,
            interest_amount,
            principal_amount,
            debit_account_id,
            principal_receivable_account_id,
            interest_receivable_account_id,
            tx_ref,
        }: RecordPaymentParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("external_id", tx_ref);
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("interest_amount", interest_amount);
        params.insert("principal_amount", principal_amount);
        params.insert("debit_account_id", debit_account_id);
        params.insert(
            "principal_receivable_account_id",
            principal_receivable_account_id,
        );
        params.insert(
            "interest_receivable_account_id",
            interest_receivable_account_id,
        );
        params.insert("effective", chrono::Utc::now().date_naive());

        params
    }
}

pub struct RecordPayment;

impl RecordPayment {
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
                .entry_type("'RECORD_PAYMENT_DR'")
                .currency("params.currency")
                .account_id("params.debit_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.principal_amount + params.interest_amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_PAYMENT_PRINCIPAL_CR'")
                .currency("params.currency")
                .account_id("params.principal_receivable_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.principal_amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_PAYMENT_INTEREST_CR'")
                .currency("params.currency")
                .account_id("params.interest_receivable_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.interest_amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = RecordPaymentParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RECORD_PAYMENT_CODE)
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
