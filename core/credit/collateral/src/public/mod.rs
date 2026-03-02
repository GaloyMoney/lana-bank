mod event;

use serde::{Deserialize, Serialize};

use money::Satoshis;

pub use event::CoreCreditCollateralEvent;

use crate::{Liquidation, LiquidationId};

use super::{Collateral, CollateralAdjustment, CollateralId, SecuredLoanId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PublicCollateral {
    pub id: CollateralId,
    pub secured_loan_id: SecuredLoanId,
    pub amount: Satoshis,
    pub adjustment: Option<CollateralAdjustment>,
}

impl From<&Collateral> for PublicCollateral {
    fn from(entity: &Collateral) -> Self {
        PublicCollateral {
            id: entity.id,
            secured_loan_id: entity.secured_loan_id,
            amount: entity.amount,
            adjustment: entity.last_adjustment(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PublicLiquidation {
    pub id: LiquidationId,
    pub collateral_id: CollateralId,
    pub secured_loan_id: SecuredLoanId,
    pub total_received: Satoshis,
    pub total_sent: Satoshis,
    pub adjustment: Option<CollateralAdjustment>,
}

impl From<&Liquidation> for PublicLiquidation {
    fn from(entity: &Liquidation) -> Self {
        Self {
            id: entity.id,
            secured_loan_id: entity.secured_loan_id,
            amount: entity.amount,
            adjustment: entity.last_adjustment(),
        }
    }
}
