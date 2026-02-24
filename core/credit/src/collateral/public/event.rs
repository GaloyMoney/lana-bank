use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use money::{Satoshis, UsdCents};

use crate::{
    primitives::{LedgerTxId, LiquidationId, PaymentId},
    public::PublicCollateral,
};

use crate::collateral::primitives::SecuredLoanId;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditCollateralEvent {
    CollateralUpdated {
        entity: PublicCollateral,
    },
    LiquidationCollateralSentOut {
        liquidation_id: LiquidationId,
        secured_loan_id: SecuredLoanId,
        amount: Satoshis,
        ledger_tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    LiquidationProceedsReceived {
        liquidation_id: LiquidationId,
        secured_loan_id: SecuredLoanId,
        amount: UsdCents,
        payment_id: PaymentId,
        ledger_tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
        effective: chrono::NaiveDate,
    },
    LiquidationCompleted {
        liquidation_id: LiquidationId,
        secured_loan_id: SecuredLoanId,
    },
}
