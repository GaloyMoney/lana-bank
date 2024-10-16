use serde::{Deserialize, Serialize};

use crate::{
    primitives::{CollateralAction, LedgerAccountId, LedgerTxId, Satoshis, UsdCents},
    terms::InterestPeriod,
};

use super::{cala::graphql::*, error::*, CustomerLedgerAccountIds};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityAccountIds {
    pub facility_account_id: LedgerAccountId,
    pub disbursed_receivable_account_id: LedgerAccountId,
    pub collateral_account_id: LedgerAccountId,
    pub interest_receivable_account_id: LedgerAccountId,
    pub interest_account_id: LedgerAccountId,
}

impl CreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            facility_account_id: LedgerAccountId::new(),
            disbursed_receivable_account_id: LedgerAccountId::new(),
            collateral_account_id: LedgerAccountId::new(),
            interest_receivable_account_id: LedgerAccountId::new(),
            interest_account_id: LedgerAccountId::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityBalance {
    pub facility: UsdCents,
    pub collateral: Satoshis,
    pub disbursed: UsdCents,
    pub disbursed_receivable: UsdCents,
    pub interest: UsdCents,
    pub accrued_interest_receivable: UsdCents,
    pub pending_interest_receivable: UsdCents,
}

impl TryFrom<credit_facility_balance::ResponseData> for CreditFacilityBalance {
    type Error = LedgerError;

    fn try_from(data: credit_facility_balance::ResponseData) -> Result<Self, Self::Error> {
        Ok(CreditFacilityBalance {
            facility: data
                .facility
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            disbursed: data
                .total_disbursed
                .map(|b| UsdCents::try_from_usd(b.settled.dr_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            disbursed_receivable: data
                .disbursed_receivable
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            interest: data
                .total_interest
                .map(|b| UsdCents::try_from_usd(b.settled.dr_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            accrued_interest_receivable: data
                .interest_receivable
                .clone()
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            pending_interest_receivable: data
                .interest_receivable
                .map(|b| UsdCents::try_from_usd(b.pending.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            collateral: data
                .collateral
                .map(|b| Satoshis::try_from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(Satoshis::ZERO))?,
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

#[derive(Debug, Clone)]
pub struct CreditFacilityRepayment {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
    pub amounts: CreditFacilityPaymentAmounts,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreditFacilityPaymentAmounts {
    pub interest: UsdCents,
    pub disbursement: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestIncurrence {
    pub interest: UsdCents,
    pub period: InterestPeriod,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrual {
    pub interest: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityCompletion {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub collateral: Satoshis,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
}
