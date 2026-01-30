use core_credit_terms::TermValues;
use core_money::UsdCents;
use core_price::PriceOfOneBTC;

use crate::ledger::{CreditFacilityBalanceSummary, PendingCreditFacilityBalanceSummary};

pub trait TermValuesExt {
    fn is_disbursal_allowed(
        &self,
        balance: CreditFacilityBalanceSummary,
        amount: UsdCents,
        price: PriceOfOneBTC,
    ) -> bool;

    fn is_proposal_completion_allowed(
        &self,
        balance: PendingCreditFacilityBalanceSummary,
        price: PriceOfOneBTC,
    ) -> bool;
}

impl TermValuesExt for TermValues {
    fn is_disbursal_allowed(
        &self,
        balance: CreditFacilityBalanceSummary,
        amount: UsdCents,
        price: PriceOfOneBTC,
    ) -> bool {
        let cvl = balance.with_added_disbursal(amount).current_cvl(price);
        cvl >= self.margin_call_cvl
    }

    fn is_proposal_completion_allowed(
        &self,
        balance: PendingCreditFacilityBalanceSummary,
        price: PriceOfOneBTC,
    ) -> bool {
        let total = balance.current_cvl(price);
        total >= self.margin_call_cvl
    }
}
