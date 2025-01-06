use crate::primitives::{Satoshis, UsdCents};

use super::{cala::graphql::*, error::*};

pub use crate::credit_facility::ledger::{
    CreditFacilityAccountIds, CreditFacilityActivation, CreditFacilityCollateralUpdate,
    CreditFacilityCompletion, CreditFacilityInterestAccrual, CreditFacilityInterestIncurrence,
    CreditFacilityLedgerBalance, CreditFacilityPaymentAmounts, CreditFacilityRepayment,
    DisbursalData,
};

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
