use rust_decimal::{Decimal, RoundingStrategy};

use crate::primitives::CVLPct;
use money::{Satoshis, UsdCents};
use core_price::PriceOfOneBTC;

pub struct LiquidationPayment;

impl LiquidationPayment {
    const UNIT_FEE_FACTOR: Decimal = Decimal::ONE;

    pub fn repay_amount(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        collateral: Satoshis,
    ) -> UsdCents {
        let target_ratio = match target_cvl {
            CVLPct::Finite(pct) => pct / Decimal::from(100u64),
            CVLPct::Infinite => return UsdCents::ZERO,
        };

        let outstanding_usd = outstanding.to_usd();
        let collateral_usd = price.sats_to_cents_round_down(collateral).to_usd();

        let repay_usd = ((outstanding_usd * target_ratio - collateral_usd)
            / (target_ratio - Self::UNIT_FEE_FACTOR))
            .max(Decimal::ZERO)
            .round_dp_with_strategy(2, RoundingStrategy::AwayFromZero);

        UsdCents::try_from_usd(repay_usd).expect("repay amount must be in whole cents")
    }
}

#[cfg(test)]
mod test {
    use money::{Satoshis, UsdCents};
    use core_price::PriceOfOneBTC;

    use crate::{CVLPct, LiquidationPayment};

    #[test]
    fn calculate() {
        let amount = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);

        let res = LiquidationPayment::repay_amount(amount, price, target_cvl, collateral);
        assert_eq!(res, UsdCents::from(1_875_000));
    }

    #[test]
    fn calculate_with_infinite_cvl() {
        let amount = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::Infinite;
        let collateral = Satoshis::from(100_000_000);

        let res = LiquidationPayment::repay_amount(amount, price, target_cvl, collateral);
        assert_eq!(res, UsdCents::ZERO);
    }
}
