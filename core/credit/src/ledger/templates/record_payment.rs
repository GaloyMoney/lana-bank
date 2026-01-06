use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const RECORD_PAYMENT_CODE: &str = "RECORD_PAYMENT";

#[derive(Debug)]
pub struct RecordPaymentParams {
    pub journal_id: JournalId,
    pub currency: Currency,
    pub amount: Decimal,
    pub payment_source_account_id: CalaAccountId,
    pub payment_holding_account_id: CalaAccountId,
    pub uncovered_outstanding_account_id: CalaAccountId,
    pub payments_made_omnibus_account_id: CalaAccountId,
    pub tx_ref: String,
    pub effective: chrono::NaiveDate,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
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
                .name("amount")
                .r#type(ParamDataType::Decimal)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("payment_source_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("payment_holding_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("uncovered_outstanding_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("payments_made_omnibus_account_id")
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
impl From<RecordPaymentParams> for Params {
    fn from(
        RecordPaymentParams {
            journal_id,
            currency,
            amount,
            payment_source_account_id,
            payment_holding_account_id,
            uncovered_outstanding_account_id,
            payments_made_omnibus_account_id,
            tx_ref,
            effective,
            initiated_by,
        }: RecordPaymentParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("external_id", tx_ref);
        params.insert("journal_id", journal_id);
        params.insert("currency", currency);
        params.insert("amount", amount);
        params.insert("payment_source_account_id", payment_source_account_id);
        params.insert("payment_holding_account_id", payment_holding_account_id);
        params.insert(
            "uncovered_outstanding_account_id",
            uncovered_outstanding_account_id,
        );
        params.insert(
            "payments_made_omnibus_account_id",
            payments_made_omnibus_account_id,
        );
        params.insert("effective", effective);
        params.insert(
            "meta",
            serde_json::json!({
                "initiated_by": initiated_by,
            }),
        );

        params
    }
}

pub struct RecordPayment;

impl RecordPayment {
    #[record_error_severity]
    #[instrument(name = "ledger.record_payment.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .external_id("params.external_id")
            .metadata("params.meta")
            .description("'Record a payment received'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_PAYMENT_DR'")
                .currency("params.currency")
                .account_id("params.payment_source_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_PAYMENT_CR'")
                .currency("params.currency")
                .account_id("params.payment_holding_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_UNCOVERED_DRAWDOWN_FROM_PAYMENT_DR'")
                .currency("params.currency")
                .account_id("params.uncovered_outstanding_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RECORD_UNCOVERED_DRAWDOWN_FROM_PAYMENT_CR'")
                .currency("params.currency")
                .account_id("params.payments_made_omnibus_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
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
