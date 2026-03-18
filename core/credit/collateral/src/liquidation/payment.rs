use rust_decimal::{Decimal, RoundingStrategy};

use core_credit_terms::CVLPct;
use core_price::PriceOfOneBTC;
use money::{CalculationAmount, Satoshis, Usd, UsdCents};

#[derive(Debug, Clone)]
pub struct LiquidationPaymentAmounts {
    pub to_liquidate: Satoshis,
    pub to_receive: UsdCents,
    pub target_cvl: CVLPct,
    /// The backend Bitcoin price (USD cents per BTC) used for calculations
    pub price: PriceOfOneBTC,
}

impl LiquidationPaymentAmounts {
    const UNIT_FEE_FACTOR: Decimal = Decimal::ONE;

    const ZERO: Self = Self {
        to_liquidate: Satoshis::ZERO,
        to_receive: UsdCents::ZERO,
        target_cvl: CVLPct::ZERO,
        price: PriceOfOneBTC::ZERO,
    };

    pub fn calculate(
        outstanding: UsdCents,
        collateral: Satoshis,
        price: PriceOfOneBTC,
        to_receive: Option<UsdCents>,
        to_liquidate: Option<Satoshis>,
        target_cvl: Option<CVLPct>,
    ) -> Option<Self> {
        match (to_receive, target_cvl, to_liquidate) {
            (Some(to_receive), Some(target_cvl), None) => {
                Some(Self::calculate_amount_to_liquidate(
                    outstanding,
                    price,
                    target_cvl,
                    collateral,
                    to_receive,
                ))
            }
            (None, Some(target_cvl), Some(to_liquidate)) => {
                Some(Self::calculate_amount_to_receive(
                    outstanding,
                    price,
                    target_cvl,
                    collateral,
                    to_liquidate,
                ))
            }
            (Some(to_receive), None, Some(to_liquidate)) => Some(Self::calculate_target_cvl(
                outstanding,
                price,
                collateral,
                to_receive,
                to_liquidate,
            )),
            (None, Some(target_cvl), None) => Some(Self::calculate_optimal_payment(
                outstanding,
                price,
                target_cvl,
                collateral,
            )),
            _ => None,
        }
    }

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

        let collateral_calc =
            CalculationAmount::<Usd>::from(price.sats_to_cents_round_down(collateral));
        let new_outstanding = outstanding - to_receive;

        let to_liquidate_cents = (collateral_calc - new_outstanding.to_calc() * target_ratio)
            .max(CalculationAmount::ZERO)
            .round_with(RoundingStrategy::AwayFromZero);
        let to_liquidate = price.cents_to_sats_round_up(to_liquidate_cents);

        Self {
            to_liquidate,
            to_receive,
            target_cvl,
            price,
        }
    }

    /// Calculates the effective liquidation price from `to_liquidate` and `to_receive`.
    ///
    /// Effective price is the USD cents per BTC that was actually applied,
    /// calculated as `to_receive / to_liquidate` (in BTC units).
    ///
    /// Returns `None` if `to_liquidate` is zero to avoid division by zero.
    pub fn effective_liquidation_price(&self) -> Option<PriceOfOneBTC> {
        if self.to_liquidate == Satoshis::ZERO {
            None
        } else {
            let effective_price_cents = CalculationAmount::<Usd>::from_major(
                self.to_receive.to_major() / self.to_liquidate.to_major(),
            )
            .round_with(RoundingStrategy::AwayFromZero);

            Some(PriceOfOneBTC::new(effective_price_cents))
        }
    }

    /// Calculates the liquidation premium percentage.
    ///
    /// Returns `None` if effective price cannot be calculated (e.g.,
    /// zero `to_liquidate`) or if `price` is zero.
    pub fn liquidation_premium_pct(&self) -> Option<Decimal> {
        if self.price == PriceOfOneBTC::ZERO {
            None
        } else {
            let effective = self.effective_liquidation_price()?.into_inner().to_usd();
            let price = self.price.into_inner().to_usd();
            Some((effective - price) / price * Decimal::from(100))
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

        let new_collateral_calc = CalculationAmount::<Usd>::from(
            price.sats_to_cents_round_down(collateral - to_liquidate),
        );

        let to_receive = (outstanding.to_calc() - new_collateral_calc / target_ratio)
            .max(CalculationAmount::ZERO)
            .round_with(RoundingStrategy::AwayFromZero);

        Self {
            to_liquidate,
            to_receive,
            target_cvl,
            price,
        }
    }

    /// Calculates expected payment given `target_cvl` and current
    /// `outstanding` amount, current `price` and current `collateral`
    /// amount.
    pub fn calculate_optimal_payment(
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

        let outstanding_calc = outstanding.to_calc();
        let collateral_calc =
            CalculationAmount::<Usd>::from(price.sats_to_cents_round_down(collateral));

        let to_receive = (outstanding_calc * target_ratio - collateral_calc)
            / (target_ratio - Self::UNIT_FEE_FACTOR);
        let to_receive = to_receive
            .max(CalculationAmount::ZERO)
            .round_with(RoundingStrategy::AwayFromZero);
        let to_liquidate = price.cents_to_sats_round_up(to_receive);

        Self {
            to_liquidate,
            to_receive,
            target_cvl,
            price,
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
            price,
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

    use crate::{CVLPct, LiquidationPaymentAmounts};

    #[test]
    fn to_liquidate_from_to_receive() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let to_receive = UsdCents::from(1_875_000);

        let res = LiquidationPaymentAmounts::calculate_amount_to_liquidate(
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

        let res = LiquidationPaymentAmounts::calculate_amount_to_receive(
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

        let res = LiquidationPaymentAmounts::calculate_target_cvl(
            outstanding,
            price,
            collateral,
            to_receive,
            to_liquidate,
        );
        assert_eq!(res.target_cvl, CVLPct::new(140));
    }

    #[test]
    fn liquidation_payment_effective_price_and_premium() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);

        let res = LiquidationPaymentAmounts::calculate_optimal_payment(
            outstanding,
            price,
            target_cvl,
            collateral,
        );

        // Verify price field is correctly set
        assert_eq!(res.price, price);

        // Verify effective price is calculated from to_liquidate and to_receive
        // For standard liquidations, effective price should equal the backend price
        let effective_price = res
            .effective_liquidation_price()
            .expect("should have effective price");
        assert_eq!(effective_price, price);

        // Verify premium is 0% for standard liquidations
        let premium = res.liquidation_premium_pct().expect("should have premium");
        assert_eq!(premium, rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn calculate_amount_to_liquidate_effective_price_and_premium() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let to_receive = UsdCents::from(1_875_000);

        let res = LiquidationPaymentAmounts::calculate_amount_to_liquidate(
            outstanding,
            price,
            target_cvl,
            collateral,
            to_receive,
        );

        assert_eq!(res.price, price);

        // Verify effective price is calculated from to_liquidate and to_receive
        let effective_price = res
            .effective_liquidation_price()
            .expect("should have effective price");
        assert_eq!(effective_price, price);

        // Verify premium is 0% for standard liquidations
        let premium = res.liquidation_premium_pct().expect("should have premium");
        assert_eq!(premium, rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn calculate_amount_to_receive_effective_price_and_premium() {
        let outstanding = UsdCents::from(5_000_000);
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));
        let target_cvl = CVLPct::new(140);
        let collateral = Satoshis::from(100_000_000);
        let to_liquidate = Satoshis::from(30_000_000);

        let res = LiquidationPaymentAmounts::calculate_amount_to_receive(
            outstanding,
            price,
            target_cvl,
            collateral,
            to_liquidate,
        );

        assert_eq!(res.price, price);

        // Verify effective price is calculated from to_liquidate and to_receive
        let effective_price = res
            .effective_liquidation_price()
            .expect("should have effective price");
        assert_eq!(effective_price, price);

        // Verify premium is 0% for standard liquidations
        let premium = res.liquidation_premium_pct().expect("should have premium");
        assert_eq!(premium, rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn calculate_target_cvl_effective_price_and_premium() {
        let price = PriceOfOneBTC::new(UsdCents::from(6_250_000));

        // Use values that intentionally create a premium with clean division:
        // to_liquidate = 25,000,000 sats = 0.25 BTC
        // to_receive = 2,000,000 cents
        // effective_price = 2,000,000 / 0.25 = 8,000,000 cents/BTC (exact)
        // premium = (8,000,000 / 6,250,000 - 1) * 100 = 28%
        let to_liquidate = Satoshis::from(25_000_000);
        let to_receive = UsdCents::from(2_000_000);

        let outstanding = UsdCents::from(5_000_000);
        let collateral = Satoshis::from(100_000_000);

        let res = LiquidationPaymentAmounts::calculate_target_cvl(
            outstanding,
            price,
            collateral,
            to_receive,
            to_liquidate,
        );

        assert_eq!(res.price, price);

        // Verify effective price is calculated from to_liquidate and to_receive
        // Expected: 2,000,000 cents / 0.25 BTC = 8,000,000 cents/BTC
        let effective_price = res
            .effective_liquidation_price()
            .expect("should have effective price");
        assert_eq!(
            effective_price,
            PriceOfOneBTC::new(UsdCents::from(8_000_000))
        );

        // Verify premium is non-zero: (8,000,000 / 6,250,000 - 1) * 100 = 28%
        let premium = res.liquidation_premium_pct().expect("should have premium");
        assert_eq!(premium, rust_decimal::Decimal::new(28, 0)); // 28%

        // Verify target CVL is still calculated correctly
        // new_collateral = 75,000,000 sats = 4,687,500 cents (at 6,250,000 cents/BTC)
        // new_outstanding = 3,000,000 cents
        // target_cvl = 4,687,500 / 3,000,000 * 100 = 156.25%
        assert_eq!(
            res.target_cvl,
            CVLPct::from_loan_amounts(UsdCents::from(4_687_500), UsdCents::from(3_000_000))
        );
    }

    #[test]
    fn effective_price_returns_none_for_zero_to_liquidate() {
        // Create a payment with zero to_liquidate
        let payment = LiquidationPaymentAmounts {
            to_liquidate: Satoshis::ZERO,
            to_receive: UsdCents::from(100_000),
            target_cvl: CVLPct::new(140),
            price: PriceOfOneBTC::new(UsdCents::from(6_250_000)),
        };

        // Should return None to avoid division by zero
        assert!(payment.effective_liquidation_price().is_none());
        assert!(payment.liquidation_premium_pct().is_none());
    }

    #[test]
    fn premium_returns_none_for_zero_price() {
        // Create a payment with zero price
        let payment = LiquidationPaymentAmounts {
            to_liquidate: Satoshis::from(100_000),
            to_receive: UsdCents::from(100_000),
            target_cvl: CVLPct::new(140),
            price: PriceOfOneBTC::ZERO,
        };

        // effective_liquidation_price should still work (calculated from to_liquidate/to_receive)
        assert!(payment.effective_liquidation_price().is_some());

        // But premium should return None due to zero price (division by zero)
        assert!(payment.liquidation_premium_pct().is_none());
    }
}
