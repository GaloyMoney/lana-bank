// Re-export balance summary types from terms crate
pub use core_credit_terms::balance_summary::{
    CreditFacilityBalanceSummary, PendingCreditFacilityBalanceSummary,
};

#[cfg(test)]
mod test {
    #![allow(clippy::inconsistent_digit_grouping)]

    use core_money::{Satoshis, UsdCents};
    use core_price::PriceOfOneBTC;
    use rust_decimal::Decimal;

    use crate::{CVLPct, CollateralizationRatio};

    use super::*;

    #[test]
    fn current_cvl_returns_infinite_when_no_disbursals() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::ZERO,

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let price = PriceOfOneBTC::new(UsdCents::from(100_000_00));
        assert_eq!(balances.current_cvl(price), CVLPct::Infinite);
    }

    #[test]
    fn current_cvl_returns_non_zero_amount_when_disbursals_with_outstanding() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::from(1),

            not_yet_due_disbursed_outstanding: UsdCents::ONE,
            due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let price = PriceOfOneBTC::new(UsdCents::from(100_000_00));
        assert_ne!(balances.current_cvl(price), CVLPct::ZERO);
        assert_ne!(balances.current_cvl(price), CVLPct::Infinite);
    }

    #[test]
    fn current_cvl_returns_infinite_when_disbursals_with_no_outstanding() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::from(1),

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let price = PriceOfOneBTC::new(UsdCents::from(100_000_00));
        assert_eq!(balances.current_cvl(price), CVLPct::Infinite);
    }

    #[test]
    fn current_collateralization_ratio_when_no_disbursals() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::ZERO,
            due_disbursed_outstanding: UsdCents::ZERO,

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        assert_eq!(
            balances.current_collateralization_ratio(),
            CollateralizationRatio::Infinite
        );
    }

    #[test]
    fn current_collateralization_ratio_when_disbursals() {
        let balances = CreditFacilityBalanceSummary {
            collateral: Satoshis::from(100),
            facility: UsdCents::from(2),
            disbursed: UsdCents::from(1),
            due_disbursed_outstanding: UsdCents::from(1),

            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,

            facility_remaining: UsdCents::from(1),
            interest_posted: UsdCents::from(1),
            payments_unapplied: UsdCents::ZERO,
        };

        let collateral = Decimal::from(balances.collateral().into_inner());
        let expected =
            collateral / Decimal::from(balances.total_outstanding_payable().into_inner());
        assert_eq!(
            balances.current_collateralization_ratio(),
            CollateralizationRatio::Finite(expected)
        );
    }
}
