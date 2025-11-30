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
    pub amount: UsdCents,
    pub price: PriceOfOneBTC,
    pub target_cvl: CVLPct,
    pub collateral: Satoshis,
}

impl LiquidationPayment {
    pub fn new(
        amount: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        collateral: Satoshis,
    ) -> Self {
        Self {
            amount,
            price,
            target_cvl,
            collateral,
        }
    }

    pub fn calculate(&self) -> UsdCents {
        let target_ratio = match self.target_cvl {
            CVLPct::Finite(pct) => pct / Decimal::from(100u64),
            CVLPct::Infinite => return UsdCents::ZERO,
        };
        let loan_usd = self.amount.to_usd();

        let collateral_usd = self
            .price
            .sats_to_cents_round_down(self.collateral)
            .to_usd();

        let liquidation_fee = Decimal::from(105u64) / Decimal::from(100u64);

        // repay_amount  = (loan * target_cvl - collateral * trigger_price) / (target_cvl - liquidation_fee)
        let repay_usd = ((loan_usd * target_ratio - collateral_usd)
            / (target_ratio - liquidation_fee))
            .max(Decimal::ZERO)
            .round_dp_with_strategy(2, RoundingStrategy::AwayFromZero);

        UsdCents::try_from_usd(repay_usd).expect("repay amount must be in whole cents")
    }
}

mod test {

    use super::*;

    #[test]
    fn test_calculate() {
        let amount = UsdCents::from(50_000_00);
        let price = PriceOfOneBTC::new(UsdCents::from(62_500_00));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let liquidation_payment = LiquidationPayment::new(amount, price, target_cvl, collateral);
        let amount = liquidation_payment.calculate();
        assert_eq!(amount, UsdCents::from(21_428_58));
    }

    #[test]

    fn test_calculate_with_infinite_cvl() {
        let amount = UsdCents::from(50_000_00);
        let price = PriceOfOneBTC::new(UsdCents::from(62_500_00));
        let target_cvl = CVLPct::Infinite;
        let collateral = Satoshis::from(100_000_000);
        let liquidation_payment = LiquidationPayment::new(amount, price, target_cvl, collateral);
        let amount = liquidation_payment.calculate();
        assert_eq!(amount, UsdCents::ZERO);
    }
}
