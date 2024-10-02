use serde::{Deserialize, Serialize};

use crate::primitives::{CollateralAction, LedgerAccountId, LedgerTxId, Satoshis, UsdCents};

use super::{cala::graphql::*, error::*, CustomerLedgerAccountIds};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityAccountIds {
    pub facility_account_id: LedgerAccountId,
    pub disbursed_receivable_account_id: LedgerAccountId,
    pub collateral_account_id: LedgerAccountId,
}

impl CreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            facility_account_id: LedgerAccountId::new(),
            disbursed_receivable_account_id: LedgerAccountId::new(),
            collateral_account_id: LedgerAccountId::new(),
        }
    }
}

pub struct CreditFacilityBalance {
    pub facility: UsdCents,
}

impl TryFrom<credit_facility_balance::ResponseData> for CreditFacilityBalance {
    type Error = LedgerError;

    fn try_from(data: credit_facility_balance::ResponseData) -> Result<Self, Self::Error> {
        Ok(CreditFacilityBalance {
            facility: data
                .facility
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
        })
    }
}

impl CreditFacilityBalance {
    pub fn check_disbursement_amount(&self, amount: UsdCents) -> Result<(), LedgerError> {
        if amount > self.facility {
            return Err(LedgerError::DisbursementAmountTooLarge(
                amount,
                self.facility,
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreditFacilityCollateralUpdate {
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub abs_diff: Satoshis,
    pub action: CollateralAction,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityApprovalData {
    pub facility: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
}
