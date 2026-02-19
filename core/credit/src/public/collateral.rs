use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

pub use crate::collateral::CollateralAdjustment;
use crate::collateral::primitives::{PendingSecuredLoanId, SecuredLoanId};
use crate::{
    collateral::Collateral,
    primitives::{CollateralId, Satoshis},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicCollateral {
    pub id: CollateralId,
    pub secured_loan_id: SecuredLoanId,
    pub pending_secured_loan_id: PendingSecuredLoanId,
    pub amount: Satoshis,
    pub adjustment: Option<CollateralAdjustment>,
}

impl From<&Collateral> for PublicCollateral {
    fn from(entity: &Collateral) -> Self {
        PublicCollateral {
            id: entity.id,
            secured_loan_id: entity.secured_loan_id,
            pending_secured_loan_id: entity.pending_secured_loan_id,
            amount: entity.amount,
            adjustment: entity.last_adjustment(),
        }
    }
}
