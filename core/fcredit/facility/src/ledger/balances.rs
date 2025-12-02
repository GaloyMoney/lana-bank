use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_price::PriceOfOneBTC;
use credit_terms::CVLPct;

use crate::primitives::{CollateralizationRatio, Satoshis, UsdCents};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PendingCreditFacilityBalanceSummary {
    facility: UsdCents,
    collateral: Satoshis,
}

impl PendingCreditFacilityBalanceSummary {
    pub fn new(facility: UsdCents, collateral: Satoshis) -> Self {
        Self {
            collateral,
            facility,
        }
    }

    pub fn collateral(&self) -> Satoshis {
        self.collateral
    }

    pub fn current_collateralization_ratio(&self) -> CollateralizationRatio {
        if self.facility.is_zero() {
            return CollateralizationRatio::Infinite;
        }

        let amount = Decimal::from(self.facility.into_inner());
        let collateral = Decimal::from(self.collateral.into_inner());

        CollateralizationRatio::Finite(collateral / amount)
    }

    pub fn facility_amount_cvl(&self, price: PriceOfOneBTC) -> CVLPct {
        CVLData::new(self.collateral, self.facility).cvl(price)
    }
}

#[derive(Clone, Debug)]
struct CVLData {
    amount: UsdCents,
    collateral: Satoshis,
}

impl CVLData {
    fn new(collateral: Satoshis, amount: UsdCents) -> Self {
        Self { collateral, amount }
    }

    fn cvl(&self, price: PriceOfOneBTC) -> CVLPct {
        let collateral_value = price.sats_to_cents_round_down(self.collateral);
        if collateral_value == UsdCents::ZERO {
            CVLPct::ZERO
        } else {
            CVLPct::from_loan_amounts(collateral_value, self.amount)
        }
    }
}
