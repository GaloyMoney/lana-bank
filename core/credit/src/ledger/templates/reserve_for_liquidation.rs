use rust_decimal::Decimal;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    tx_template::{Params, error::TxTemplateError, *},
    *,
};

use crate::{ledger::error::*, primitives::CalaAccountId};

pub const RESERVE_FOR_LIQUIDATION_CODE: &str = "RESERVE_FOR_LIQUIDATION";

#[derive(Debug)]
pub struct ReserveForLiquidationParams {
    pub journal_id: JournalId,
    pub amount: Decimal,
    pub liquidation_omnibus_account_id: CalaAccountId,
    pub facility_liquidation_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
    pub initiated_by: core_accounting::LedgerTransactionInitiator,
}

impl ReserveForLiquidationParams {
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
                .name("liquidation_omnibus_account_id")
                .r#type(ParamDataType::Uuid)
                .build()
                .unwrap(),
            NewParamDefinition::builder()
                .name("facility_liquidation_account_id")
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
impl From<ReserveForLiquidationParams> for Params {
    fn from(
        ReserveForLiquidationParams {
            journal_id,
            amount,
            liquidation_omnibus_account_id,
            facility_liquidation_account_id,
            effective,
            initiated_by,
        }: ReserveForLiquidationParams,
    ) -> Self {
        let mut params = Self::default();
        params.insert("journal_id", journal_id);
        params.insert("amount", amount);
        params.insert(
            "liquidation_omnibus_account_id",
            liquidation_omnibus_account_id,
        );
        params.insert(
            "facility_liquidation_account_id",
            facility_liquidation_account_id,
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

pub struct ReserveForLiquidation;

impl ReserveForLiquidation {
    #[record_error_severity]
    #[instrument(name = "ledger.reserve_for_liquidation.init", skip_all)]
    pub async fn init(ledger: &CalaLedger) -> Result<(), CreditLedgerError> {
        let tx_input = NewTxTemplateTransaction::builder()
            .journal_id("params.journal_id")
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Reserve an outstanding amount to be repaid via liquidation'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            NewTxTemplateEntry::builder()
                .entry_type("'RESERVE_FOR_LIQUIDATION_CR'")
                .currency("'USD'")
                .account_id("params.facility_liquidation_account_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
            NewTxTemplateEntry::builder()
                .entry_type("'RESERVE_FOR_LIQUIDATION_DR'")
                .currency("'USD'")
                .account_id("params.liquidation_omnibus_account_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.amount")
                .build()
                .expect("Couldn't build entry"),
        ];

        let params = ReserveForLiquidationParams::defs();
        let template = NewTxTemplate::builder()
            .id(TxTemplateId::new())
            .code(RESERVE_FOR_LIQUIDATION_CODE)
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
