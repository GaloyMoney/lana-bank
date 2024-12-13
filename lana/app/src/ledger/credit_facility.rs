use chrono::{DateTime, Utc};

use crate::{
    primitives::{CollateralAction, LedgerTxId, Satoshis, UsdCents},
    terms::InterestPeriod,
};

use super::{cala::graphql::*, error::*, CustomerLedgerAccountIds};

pub use crate::credit_facility::ledger::{CreditFacilityAccountIds, CreditFacilityLedgerBalance};

impl TryFrom<credit_facility_ledger_balance::ResponseData> for CreditFacilityLedgerBalance {
    type Error = LedgerError;

    fn try_from(data: credit_facility_ledger_balance::ResponseData) -> Result<Self, Self::Error> {
        Ok(CreditFacilityLedgerBalance {
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
            interest_receivable: data
                .interest_receivable
                .clone()
                .map(|b| UsdCents::try_from_usd(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(UsdCents::ZERO))?,
            collateral: data
                .collateral
                .map(|b| Satoshis::try_from_btc(b.settled.normal_balance.units))
                .unwrap_or_else(|| Ok(Satoshis::ZERO))?,
        })
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
pub struct CreditFacilityActivationData {
    pub facility: UsdCents,
    pub structuring_fee: UsdCents,
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
    pub disbursal: UsdCents,
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
    pub accrued_at: DateTime<Utc>,
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
