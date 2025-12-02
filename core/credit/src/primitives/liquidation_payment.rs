use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::CVLPct;
use core_money::{Satoshis, UsdCents};
use core_price::PriceOfOneBTC;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct LiquidationPayment {
    outstanding: UsdCents,
    price: PriceOfOneBTC,
    target_cvl: CVLPct,
    collateral: Satoshis,
}

impl LiquidationPayment {
    const UNIT_FEE_FACTOR: Decimal = Decimal::ONE;

    pub const fn new(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        collateral: Satoshis,
    ) -> Self {
        Self {
            outstanding,
            price,
            target_cvl,
            collateral,
        }
    }

    pub fn repay_amount(&self) -> UsdCents {
        let target_ratio = match self.target_cvl {
            CVLPct::Finite(pct) => pct / Decimal::from(100u64),
            CVLPct::Infinite => return UsdCents::ZERO,
        };
        let outstanding_usd = self.outstanding.to_usd();

        let collateral_usd = self
            .price
            .sats_to_cents_round_down(self.collateral)
            .to_usd();

        let repay_usd = ((outstanding_usd * target_ratio - collateral_usd)
            / (target_ratio - Self::UNIT_FEE_FACTOR))
            .max(Decimal::ZERO)
            .round_dp_with_strategy(2, RoundingStrategy::AwayFromZero);

        UsdCents::try_from_usd(repay_usd).expect("repay amount must be in whole cents")
    }
}

#[cfg(test)]
mod test {
    use core_money::{Satoshis, UsdCents};
    use core_price::PriceOfOneBTC;

    use crate::{CVLPct, LiquidationPayment};

    #[test]
    fn calculate() {
        let amount = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let liquidation_payment = LiquidationPayment::new(amount, price, target_cvl, collateral);
        let amount = liquidation_payment.repay_amount();
        assert_eq!(amount, UsdCents::from(1_875_000));
    }

    #[test]

    fn calculate_with_infinite_cvl() {
        let amount = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::Infinite;
        let collateral = Satoshis::from(100_000_000);
        let liquidation_payment = LiquidationPayment::new(amount, price, target_cvl, collateral);
        let amount = liquidation_payment.repay_amount();
        assert_eq!(amount, UsdCents::ZERO);
    }
}
