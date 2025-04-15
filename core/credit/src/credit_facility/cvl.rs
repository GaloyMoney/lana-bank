use core_money::{Satoshis, UsdCents};
use core_price::PriceOfOneBTC;

use crate::terms::{CVLPct, TermValues};

#[derive(Clone)]
pub struct CVLData {
    amount: UsdCents,
    collateral: Satoshis,
}

impl CVLData {
    pub fn new(collateral: Satoshis, amount: UsdCents) -> Self {
        Self { collateral, amount }
    }

    pub fn cvl(&self, price: PriceOfOneBTC) -> CVLPct {
        let collateral_value = price.sats_to_cents_round_down(self.collateral);
        if collateral_value == UsdCents::ZERO {
            CVLPct::ZERO
        } else {
            CVLPct::from_loan_amounts(collateral_value, self.amount)
        }
    }
}

#[derive(Clone)]
pub struct FacilityCVLData {
    pub total: CVLData,
    pub disbursed: CVLData,
}

impl FacilityCVLData {
    pub fn cvl(&self, price: PriceOfOneBTC) -> FacilityCVL {
        FacilityCVL {
            total: self.total.cvl(price),
            disbursed: self.disbursed.cvl(price),
        }
    }
}

pub struct FacilityCVL {
    pub total: CVLPct,
    pub disbursed: CVLPct,
}

impl FacilityCVL {
    pub fn is_approval_allowed(&self, terms: TermValues) -> bool {
        self.total >= terms.margin_call_cvl
    }

    pub fn is_disbursal_allowed(&self, terms: TermValues) -> bool {
        let cvl = if self.disbursed.is_zero() {
            self.total
        } else {
            self.disbursed
        };

        cvl >= terms.margin_call_cvl
    }
}
