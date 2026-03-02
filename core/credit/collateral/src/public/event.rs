use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use cala_ledger::TransactionId as LedgerTxId;
use core_credit_collection::PaymentId;
use money::{Satoshis, UsdCents};

use crate::{LiquidationId, SecuredLoanId};

use super::{PublicCollateral, PublicLiquidation};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditCollateralEvent {
    CollateralUpdated {
        entity: PublicCollateral,
    },
    LiquidationCollateralSentOut {
        entity: PublicLiquidation,
    },
    // LiquidationCollateralSentOut {
    //     liquidation_id: LiquidationId,
    //     secured_loan_id: SecuredLoanId,
    //     amount: Satoshis,
    //     ledger_tx_id: LedgerTxId,
    //     recorded_at: DateTime<Utc>,
    //     effective: chrono::NaiveDate,
    // },
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
