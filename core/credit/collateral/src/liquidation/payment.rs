use rust_decimal::{Decimal, RoundingStrategy};

use core_credit_terms::CVLPct;
use core_price::PriceOfOneBTC;
use money::{Satoshis, UsdCents};

#[derive(Debug, Clone)]
pub struct LiquidationPayment {
    pub to_liquidate: Satoshis,
    pub to_receive: UsdCents,
    pub target_cvl: CVLPct,
}

impl LiquidationPayment {
    const UNIT_FEE_FACTOR: Decimal = Decimal::ONE;

    const ZERO: Self = Self {
        to_liquidate: Satoshis::ZERO,
        to_receive: UsdCents::ZERO,
        target_cvl: CVLPct::ZERO,
    };

    /// Calculates, under current `price`, current `collateral` and
    /// current `outstanding` amount, amount to liquidate so that
    /// `target_cvl` holds.
    pub fn calculate_amount_to_liquidate(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        collateral: Satoshis,
        to_receive: UsdCents,
    ) -> Self {
        let target_ratio = Self::target_ratio_or_zero(target_cvl);
        if target_ratio == Decimal::ZERO {
            return Self::ZERO;
        }

        let collateral_usd = price.sats_to_cents_round_down(collateral).to_usd();
        let new_outstanding = outstanding - to_receive;

        let to_liquidate_usd = (collateral_usd - target_ratio * new_outstanding.to_usd())
            .max(Decimal::ZERO)
            .round_dp_with_strategy(2, RoundingStrategy::AwayFromZero);

        let to_liquidate_cents = UsdCents::try_from_usd(to_liquidate_usd)
            .expect("liquidate amount must be in whole cents");
        let to_liquidate = price.cents_to_sats_round_up(to_liquidate_cents);

        Self {
            to_liquidate,
            to_receive,
            target_cvl,
        }
    }

    /// Calculates, under current `price`, current `collateral` and
    /// current `outstanding` amount, amount to receive so that
    /// `target_cvl` holds.
    pub fn calculate_amount_to_receive(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        collateral: Satoshis,
        to_liquidate: Satoshis,
    ) -> Self {
        let target_ratio = Self::target_ratio_or_zero(target_cvl);
        if target_ratio == Decimal::ZERO {
            return Self::ZERO;
        }

        let new_collateral_usd = price
            .sats_to_cents_round_down(collateral - to_liquidate)
            .to_usd();

        let repay_usd = (outstanding.to_usd() - new_collateral_usd / target_ratio)
            .max(Decimal::ZERO)
            .round_dp_with_strategy(2, RoundingStrategy::AwayFromZero);

        let to_receive =
            UsdCents::try_from_usd(repay_usd).expect("repay amount must be in whole cents");

        Self {
            to_liquidate,
            to_receive,
            target_cvl,
        }
    }

    /// Calculates expected payment given `target_cvl` and current
    /// `outstanding` amount, current `price` and current `collateral`
    /// amount.
    pub fn calculate_liquidation_payment(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        collateral: Satoshis,
    ) -> Self {
        let target_ratio = match target_cvl {
            CVLPct::Finite(pct) => pct / Decimal::from(100u64),
            CVLPct::Infinite => {
                return Self::ZERO;
            }
        };

        let outstanding_usd = outstanding.to_usd();
        let collateral_usd = price.sats_to_cents_round_down(collateral).to_usd();

        let repay_usd = ((outstanding_usd * target_ratio - collateral_usd)
            / (target_ratio - Self::UNIT_FEE_FACTOR))
            .max(Decimal::ZERO)
            .round_dp_with_strategy(2, RoundingStrategy::AwayFromZero);

        let to_receive =
            UsdCents::try_from_usd(repay_usd).expect("repay amount must be in whole cents");
        let to_liquidate = price.cents_to_sats_round_up(to_receive);

        Self {
            to_liquidate,
            to_receive,
            target_cvl,
        }
    }

    /// Calculates target CVL if, given current `outstanding` amount,
    /// current `collateral` and current `price`, `to_liquidate` were
    /// liquidated and `to_receive` were received.
    pub fn calculate_target_cvl(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        collateral: Satoshis,
        to_receive: UsdCents,
        to_liquidate: Satoshis,
    ) -> Self {
        let new_collateral_usd = price.sats_to_cents_round_down(collateral - to_liquidate);
        let new_outstanding = outstanding - to_receive;

        Self {
            to_liquidate,
            to_receive,
            target_cvl: CVLPct::from_loan_amounts(new_collateral_usd, new_outstanding),
        }
    }

    fn target_ratio_or_zero(target_cvl: CVLPct) -> Decimal {
        match target_cvl {
            CVLPct::Finite(pct) => pct / Decimal::from(100u64),
            CVLPct::Infinite => Decimal::ZERO,
        }
    }
}

#[cfg(test)]
mod test {
    use core_price::PriceOfOneBTC;
    use money::{Satoshis, UsdCents};

    use crate::{CVLPct, LiquidationPayment};

    #[test]
    fn to_liquidate_from_to_receive() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let to_receive = UsdCents::from(1_875_000);

        let res = LiquidationPayment::calculate_amount_to_liquidate(
            outstanding,
            price,
            target_cvl,
            collateral,
            to_receive,
        );
        assert_eq!(res.to_liquidate, Satoshis::from(30_000_000));
    }

    #[test]
    fn calculate_amount_to_receive() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let to_liquidate = Satoshis::from(30_000_000);

        let res = LiquidationPayment::calculate_amount_to_receive(
            outstanding,
            price,
            target_cvl,
            collateral,
            to_liquidate,
        );
        assert_eq!(res.to_receive, UsdCents::from(1_875_000));
    }

    #[test]
    fn target_cvl_from_payment_amounts() {
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));

        let to_liquidate = Satoshis::from(30_000_000);
        let to_receive = UsdCents::from(1_875_000);

        let outstanding = UsdCents::from(5_000_000);
        let collateral = Satoshis::from(100_000_000);

        let res = LiquidationPayment::calculate_target_cvl(
            outstanding,
            price,
            collateral,
            to_receive,
            to_liquidate,
        );
        assert_eq!(res.target_cvl, CVLPct::new(140));
    }
}
