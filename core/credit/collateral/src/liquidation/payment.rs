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

    pub fn to_liquidate_from_to_receive(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        collateral: Satoshis,
        to_receive: UsdCents,
    ) -> Self {
        let target_ratio = Self::target_ratio_or_zero(target_cvl);
        let collateral_usd = price.sats_to_cents_round_down(collateral).to_usd();

        let required_collateral = outstanding.to_usd() * target_ratio;
        if to_receive.to_usd() > required_collateral - collateral_usd {
            let to_liquidate = price.cents_to_sats_round_up(to_receive);
            Self {
                to_liquidate,
                to_receive,
                target_cvl,
            }
        } else {
            Self::ZERO
        }
    }

    pub fn to_receive_from_to_liquidate(
        _outstanding: UsdCents,
        price: PriceOfOneBTC,
        target_cvl: CVLPct,
        _collateral: Satoshis,
        to_liquidate: Satoshis,
    ) -> Self {
        let to_receive = price.sats_to_cents_round_down(to_liquidate);
        Self {
            to_liquidate,
            to_receive,
            target_cvl,
        }
    }

    pub fn calculate_from_target_cvl(
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

    pub fn target_cvl_from_payment_amounts(
        outstanding: UsdCents,
        price: PriceOfOneBTC,
        collateral: Satoshis,
        to_receive: UsdCents,
        _to_liquidate: Satoshis,
    ) -> CVLPct {
        let to_receive_usd = to_receive.to_usd();
        let outstanding_usd = outstanding.to_usd();
        let collateral_usd = price.sats_to_cents_round_down(collateral).to_usd();

        if to_receive_usd <= outstanding_usd && to_receive_usd != collateral_usd {
            let target_ratio =
                (to_receive_usd - collateral_usd) / (to_receive_usd - outstanding_usd);
            let cvl_pct = (target_ratio * Decimal::from(100u64))
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero);
            CVLPct::Finite(cvl_pct)
        } else if to_receive_usd > outstanding_usd {
            CVLPct::Infinite
        } else {
            CVLPct::ZERO
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

        let res = LiquidationPayment::to_liquidate_from_to_receive(
            outstanding,
            price,
            target_cvl,
            collateral,
            to_receive,
        );
        assert_eq!(res.to_liquidate, Satoshis::from(30_000_000));
    }

    #[test]
    fn to_receive_from_to_liquidate() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let to_liquidate = Satoshis::from(30_000_000);

        let res = LiquidationPayment::to_receive_from_to_liquidate(
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
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let collateral = Satoshis::from(100_000_000);
        let to_receive = UsdCents::from(1_875_000);
        let to_liquidate = Satoshis::from(30_000);

        let res = LiquidationPayment::target_cvl_from_payment_amounts(
            outstanding,
            price,
            collateral,
            to_receive,
            to_liquidate,
        );
        assert_eq!(res, CVLPct::new(140));
    }

    #[test]
    fn target_cvl_infinite_when_excess_payment() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let collateral = Satoshis::from(100_000_000);
        let to_receive = UsdCents::from(10_000_000);
        let to_liquidate = Satoshis::from(160_000);

        let res = LiquidationPayment::target_cvl_from_payment_amounts(
            outstanding,
            price,
            collateral,
            to_receive,
            to_liquidate,
        );
        assert_eq!(res, CVLPct::Infinite);
    }
}
