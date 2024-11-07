use chrono::{DateTime, Datelike, TimeZone, Utc};
use derive_builder::{Builder, UninitializedFieldError};
use rust_decimal::{prelude::*, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::error::TermsError;
use crate::primitives::{PriceOfOneBTC, Satoshis, UsdCents};

const NUMBER_OF_DAYS_IN_YEAR: u64 = 366;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    Deserialize,
    Eq,
    async_graphql::Enum,
    strum::Display,
    strum::EnumString,
)]
pub enum CollateralizationState {
    FullyCollateralized,
    UnderMarginCallThreshold,
    UnderLiquidationThreshold,
    #[default]
    NoCollateral,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct AnnualRatePct(Decimal);
async_graphql::scalar!(AnnualRatePct);

impl AnnualRatePct {
    pub fn interest_for_time_period(&self, principal: UsdCents, days: u32) -> UsdCents {
        let cents = principal.to_usd() * Decimal::from(days) * self.0
            / Decimal::from(NUMBER_OF_DAYS_IN_YEAR);

        UsdCents::from(
            cents
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }
}

impl From<Decimal> for AnnualRatePct {
    fn from(value: Decimal) -> Self {
        AnnualRatePct(value)
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CVLPct(Decimal);
async_graphql::scalar!(CVLPct);

impl fmt::Display for CVLPct {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for CVLPct {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        CVLPct(self.0 + other.0)
    }
}

impl std::ops::Sub for CVLPct {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        CVLPct(self.0 - other.0)
    }
}

impl CVLPct {
    pub const ZERO: Self = Self(dec!(0));

    pub fn new(value: u64) -> Self {
        Self(Decimal::from(value))
    }

    pub fn scale(&self, value: UsdCents) -> UsdCents {
        let cents = value.to_usd() * dec!(100) * (self.0 / dec!(100));
        UsdCents::from(
            cents
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }

    pub fn from_loan_amounts(
        collateral_value: UsdCents,
        total_outstanding_amount: UsdCents,
    ) -> Self {
        if collateral_value == UsdCents::ZERO || total_outstanding_amount == UsdCents::ZERO {
            return CVLPct::ZERO;
        }

        let ratio = (collateral_value.to_usd() / total_outstanding_amount.to_usd())
            .round_dp_with_strategy(2, RoundingStrategy::ToZero)
            * dec!(100);

        CVLPct::from(ratio)
    }

    pub fn is_significantly_lower_than(&self, other: CVLPct, buffer: CVLPct) -> bool {
        other > *self + buffer
    }

    pub fn collateralization(self, terms: TermValues) -> CollateralizationState {
        let margin_call_cvl = terms.margin_call_cvl;
        let liquidation_cvl = terms.liquidation_cvl;

        if self == CVLPct::ZERO {
            CollateralizationState::NoCollateral
        } else if self >= margin_call_cvl {
            CollateralizationState::FullyCollateralized
        } else if self >= liquidation_cvl {
            CollateralizationState::UnderMarginCallThreshold
        } else {
            CollateralizationState::UnderLiquidationThreshold
        }
    }

    pub fn collateralization_update(
        self,
        terms: TermValues,
        last_collateralization_state: CollateralizationState,
        upgrade_buffer_cvl_pct: Option<CVLPct>,
        liquidation_upgrade_blocked: bool,
    ) -> Option<CollateralizationState> {
        let calculated_collateralization = &self.collateralization(terms);

        match (last_collateralization_state, *calculated_collateralization) {
            // Redundant same state changes
            (CollateralizationState::NoCollateral, CollateralizationState::NoCollateral)
            | (
                CollateralizationState::FullyCollateralized,
                CollateralizationState::FullyCollateralized,
            )
            | (
                CollateralizationState::UnderMarginCallThreshold,
                CollateralizationState::UnderMarginCallThreshold,
            )
            | (
                CollateralizationState::UnderLiquidationThreshold,
                CollateralizationState::UnderLiquidationThreshold,
            ) => None,

            // Validated liquidation changes
            (CollateralizationState::UnderLiquidationThreshold, _) => {
                if liquidation_upgrade_blocked {
                    None
                } else {
                    Some(*calculated_collateralization)
                }
            }

            // Optionally buffered collateral upgraded change
            (
                CollateralizationState::UnderMarginCallThreshold,
                CollateralizationState::FullyCollateralized,
            ) => match upgrade_buffer_cvl_pct {
                Some(buffer) => {
                    if terms
                        .margin_call_cvl
                        .is_significantly_lower_than(self, buffer)
                    {
                        Some(*calculated_collateralization)
                    } else {
                        None
                    }
                }
                _ => Some(*calculated_collateralization),
            },

            // Valid other collateral changes
            (CollateralizationState::NoCollateral, _)
            | (CollateralizationState::FullyCollateralized, _)
            | (CollateralizationState::UnderMarginCallThreshold, _) => {
                Some(*calculated_collateralization)
            }
        }
    }

    #[cfg(test)]
    pub fn target_value_given_outstanding(&self, outstanding: UsdCents) -> UsdCents {
        let target_in_usd = self.0 / dec!(100) * outstanding.to_usd();
        UsdCents::from(
            (target_in_usd * dec!(100))
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }
}

impl From<Decimal> for CVLPct {
    fn from(value: Decimal) -> Self {
        CVLPct(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Duration {
    Months(u32),
}

impl Duration {
    pub fn expiration_date(&self, start_date: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Duration::Months(months) => start_date
                .checked_add_months(chrono::Months::new(*months))
                .expect("should return a expiration date"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InterestPeriod {
    pub interval: InterestInterval,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl InterestPeriod {
    fn new(interval: InterestInterval, start: DateTime<Utc>) -> Self {
        let end = interval.end_date_starting_at(start);
        Self {
            interval,
            start,
            end,
        }
    }

    pub fn next(&self) -> Self {
        Self::new(self.interval, self.end + chrono::Duration::seconds(1))
    }

    pub fn truncate(&self, latest_possible_end_date: DateTime<Utc>) -> Option<Self> {
        if self.start > latest_possible_end_date {
            return None;
        }

        Some(Self {
            interval: self.interval,
            start: self.start,
            end: self.end.min(latest_possible_end_date),
        })
    }

    pub fn days(&self) -> u32 {
        self.end.day() - self.start.day() + 1
    }
}

#[derive(
    Debug, async_graphql::Enum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterestInterval {
    EndOfMonth,
    EndOfDay,
}

impl InterestInterval {
    pub fn period_from(&self, start_date: DateTime<Utc>) -> InterestPeriod {
        InterestPeriod::new(*self, start_date)
    }

    fn end_date_starting_at(&self, current_date: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            InterestInterval::EndOfMonth => {
                let current_year = current_date.year();
                let current_month = current_date.month();

                let (year, month) = if current_month == 12 {
                    (current_year + 1, 1)
                } else {
                    (current_year, current_month + 1)
                };

                Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0)
                    .single()
                    .expect("should return a valid date time")
                    - chrono::Duration::seconds(1)
            }
            InterestInterval::EndOfDay => Utc
                .with_ymd_and_hms(
                    current_date.year(),
                    current_date.month(),
                    current_date.day(),
                    23,
                    59,
                    59,
                )
                .single()
                .expect("should return a valid date time"),
        }
    }
}

#[derive(Builder, Debug, Serialize, Deserialize, Clone, Copy)]
#[builder(build_fn(validate = "Self::validate", error = "TermsError"))]
pub struct TermValues {
    #[builder(setter(into))]
    pub annual_rate: AnnualRatePct,
    #[builder(setter(into))]
    pub duration: Duration,
    #[builder(setter(into))]
    pub accrual_interval: InterestInterval,
    #[builder(setter(into))]
    pub incurrence_interval: InterestInterval,
    // overdue_penalty_rate: LoanAnnualRate,
    #[builder(setter(into))]
    pub liquidation_cvl: CVLPct,
    #[builder(setter(into))]
    pub margin_call_cvl: CVLPct,
    #[builder(setter(into))]
    pub initial_cvl: CVLPct,
}

impl TermValues {
    pub fn builder() -> TermValuesBuilder {
        TermValuesBuilder::default()
    }

    pub fn required_collateral(
        &self,
        desired_principal: UsdCents,
        price: PriceOfOneBTC,
    ) -> Satoshis {
        let collateral_value = self.initial_cvl.scale(desired_principal);
        price.cents_to_sats_round_up(collateral_value)
    }
}

impl TermValuesBuilder {
    fn validate(&self) -> Result<(), TermsError> {
        let initial_cvl = self
            .initial_cvl
            .ok_or(UninitializedFieldError::new("initial_cvl"))?;
        let margin_call_cvl = self
            .margin_call_cvl
            .ok_or(UninitializedFieldError::new("margin_call_cvl"))?;
        let liquidation_cvl = self
            .liquidation_cvl
            .ok_or(UninitializedFieldError::new("liquidation_cvl"))?;

        if initial_cvl <= margin_call_cvl {
            return Err(TermsError::MarginCallAboveInitialLimit(
                margin_call_cvl,
                initial_cvl,
            ));
        }

        if margin_call_cvl <= liquidation_cvl {
            return Err(TermsError::MarginCallBelowLiquidationLimit(
                margin_call_cvl,
                liquidation_cvl,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn loan_cvl_pct_scale() {
        let cvl = CVLPct(dec!(140));
        let value = UsdCents::from(100000);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::try_from_usd(dec!(1400)).unwrap());

        let cvl = CVLPct(dec!(50));
        let value = UsdCents::from(333333);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::try_from_usd(dec!(1666.67)).unwrap());
    }

    #[test]
    fn current_cvl_from_loan_amounts() {
        let expected_cvl = CVLPct(dec!(125));
        let collateral_value = UsdCents::from(125000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct(dec!(75));
        let collateral_value = UsdCents::from(75000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);
    }

    #[test]
    fn current_cvl_for_zero_amounts() {
        let expected_cvl = CVLPct::ZERO;
        let collateral_value = UsdCents::ZERO;
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);

        let expected_cvl = CVLPct::ZERO;
        let collateral_value = UsdCents::from(75000);
        let outstanding_amount = UsdCents::ZERO;
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(cvl, expected_cvl);
    }

    #[test]
    fn cvl_is_significantly_higher() {
        let buffer = CVLPct::new(5);

        let collateral_value = UsdCents::from(125000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);

        let collateral_value = UsdCents::from(130999);
        let outstanding_amount = UsdCents::from(100000);
        let slightly_higher_cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert!(!cvl.is_significantly_lower_than(slightly_higher_cvl, buffer));

        let collateral_value = UsdCents::from(131000);
        let outstanding_amount = UsdCents::from(100000);
        let significantly_higher_cvl =
            CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert!(cvl.is_significantly_lower_than(significantly_higher_cvl, buffer));
    }

    fn terms() -> TermValues {
        TermValues::builder()
            .annual_rate(AnnualRatePct(dec!(12)))
            .duration(Duration::Months(3))
            .accrual_interval(InterestInterval::EndOfMonth)
            .incurrence_interval(InterestInterval::EndOfDay)
            .liquidation_cvl(CVLPct(dec!(105)))
            .margin_call_cvl(CVLPct(dec!(125)))
            .initial_cvl(CVLPct(dec!(140)))
            .build()
            .expect("should build a valid term")
    }

    #[test]
    fn invalid_term_values_margin_call_greater_than_initial() {
        let result = TermValues::builder()
            .annual_rate(AnnualRatePct(dec!(12)))
            .duration(Duration::Months(3))
            .accrual_interval(InterestInterval::EndOfMonth)
            .liquidation_cvl(CVLPct(dec!(105)))
            .margin_call_cvl(CVLPct(dec!(150)))
            .initial_cvl(CVLPct(dec!(140)))
            .build();

        match result.unwrap_err() {
            TermsError::MarginCallAboveInitialLimit(margin_call, initial) => {
                assert_eq!(margin_call, CVLPct(dec!(150)));
                assert_eq!(initial, CVLPct(dec!(140)));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn invalid_term_values_liquidation_greater_than_margin_call() {
        let result = TermValues::builder()
            .annual_rate(AnnualRatePct(dec!(12)))
            .duration(Duration::Months(3))
            .accrual_interval(InterestInterval::EndOfMonth)
            .liquidation_cvl(CVLPct(dec!(130)))
            .margin_call_cvl(CVLPct(dec!(125)))
            .initial_cvl(CVLPct(dec!(140)))
            .build();

        match result.unwrap_err() {
            TermsError::MarginCallBelowLiquidationLimit(margin_call, liquidation) => {
                assert_eq!(margin_call, CVLPct(dec!(125)));
                assert_eq!(liquidation, CVLPct(dec!(130)));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn invalid_term_values_margin_call_equal_to_liquidation() {
        let result = TermValues::builder()
            .annual_rate(AnnualRatePct(dec!(12)))
            .duration(Duration::Months(3))
            .accrual_interval(InterestInterval::EndOfMonth)
            .liquidation_cvl(CVLPct(dec!(125)))
            .margin_call_cvl(CVLPct(dec!(125)))
            .initial_cvl(CVLPct(dec!(140)))
            .build();

        match result.unwrap_err() {
            TermsError::MarginCallBelowLiquidationLimit(margin_call, liquidation) => {
                assert_eq!(margin_call, CVLPct(dec!(125)));
                assert_eq!(liquidation, CVLPct(dec!(125)));
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn required_collateral() {
        let price =
            PriceOfOneBTC::new(UsdCents::try_from_usd(rust_decimal_macros::dec!(1000)).unwrap());
        let terms = terms();
        let principal = UsdCents::from(100000);
        let required_collateral = terms.required_collateral(principal, price);
        let sats = Satoshis::try_from_btc(dec!(1.4)).unwrap();
        assert_eq!(required_collateral, sats);
    }

    #[test]
    fn days() {
        let start_date = "2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfMonth.period_from(start_date).days(),
            29
        );

        let start_date = "2024-12-01T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfMonth.period_from(start_date).days(),
            31
        );
    }

    #[test]
    fn truncate() {
        let start_date = "2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let period = InterestInterval::EndOfMonth.period_from(start_date);

        let latest_before_start_date = "2024-12-02T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(period.truncate(latest_before_start_date), None);

        let latest_after_start_date_before_end_date =
            "2024-12-20T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(
            period
                .truncate(latest_after_start_date_before_end_date)
                .unwrap()
                .end
                < period.end
        );

        let latest_after_end_date = "2025-01-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            period.truncate(latest_after_end_date).unwrap().end,
            period.end
        );
    }

    #[test]
    fn interest_calculation() {
        let terms = terms();
        let principal = UsdCents::try_from_usd(dec!(100)).unwrap();
        let days = 366;
        let interest = terms.annual_rate.interest_for_time_period(principal, days);
        assert_eq!(interest, UsdCents::from(1200));

        let principal = UsdCents::try_from_usd(dec!(1000)).unwrap();
        let days = 23;
        let interest = terms.annual_rate.interest_for_time_period(principal, days);
        assert_eq!(interest, UsdCents::from(755));
    }

    #[test]
    fn expiration_date() {
        let start_date = "2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let duration = Duration::Months(3);
        let expiration_date = "2025-03-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(duration.expiration_date(start_date), expiration_date);
    }

    #[test]
    fn next_period() {
        let start_date = "2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();

        let expected_next_start_date = "2024-12-04T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let day_period = InterestInterval::EndOfDay.period_from(start_date);
        assert_eq!(day_period.next().start, expected_next_start_date);

        let expected_next_start_date = "2025-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let month_period = InterestInterval::EndOfMonth.period_from(start_date);
        assert_eq!(month_period.next().start, expected_next_start_date);
    }

    #[test]
    fn end_date_starting_at_month_interval() {
        let expected_end_date = "2024-12-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap();

        let start_of_month = "2024-12-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfMonth.end_date_starting_at(start_of_month),
            expected_end_date
        );

        let middle_of_month = "2024-12-15T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfMonth.end_date_starting_at(middle_of_month),
            expected_end_date
        );

        let end_of_month = "2024-12-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfMonth.end_date_starting_at(end_of_month),
            expected_end_date
        );
    }

    #[test]
    fn end_date_starting_at_day_interval() {
        let expected_end_date = "2024-12-03T23:59:59Z".parse::<DateTime<Utc>>().unwrap();

        let start_of_day = "2024-12-03T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfDay.end_date_starting_at(start_of_day),
            expected_end_date
        );

        let middle_of_day = "2024-12-03T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfDay.end_date_starting_at(middle_of_day),
            expected_end_date
        );

        let end_of_day = "2024-12-03T23:59:59Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            InterestInterval::EndOfDay.end_date_starting_at(end_of_day),
            expected_end_date
        );
    }

    mod collateralization_update {
        use super::*;

        fn default_upgrade_buffer_cvl_pct() -> CVLPct {
            CVLPct::new(5)
        }

        fn default_terms() -> TermValues {
            TermValues::builder()
                .annual_rate(dec!(12))
                .duration(Duration::Months(3))
                .accrual_interval(InterestInterval::EndOfMonth)
                .incurrence_interval(InterestInterval::EndOfDay)
                .liquidation_cvl(dec!(105))
                .margin_call_cvl(dec!(125))
                .initial_cvl(dec!(140))
                .build()
                .expect("should build a valid term")
        }

        struct TestCVL {
            above_fully_collateralized: CVLPct,
            above_margin_called_and_buffer: CVLPct,
            above_margin_called_and_below_buffer: CVLPct,
            below_margin_called: CVLPct,
            below_liquidation: CVLPct,
        }
        fn cvl_test_values() -> TestCVL {
            let terms = default_terms();
            let upgrade_buffer_cvl = default_upgrade_buffer_cvl_pct();

            TestCVL {
                above_fully_collateralized: terms.initial_cvl + CVLPct::new(1),
                above_margin_called_and_buffer: terms.margin_call_cvl
                    + upgrade_buffer_cvl
                    + CVLPct::new(1),
                above_margin_called_and_below_buffer: terms.margin_call_cvl + upgrade_buffer_cvl
                    - CVLPct::new(1),
                below_margin_called: terms.margin_call_cvl - CVLPct::new(1),
                below_liquidation: terms.liquidation_cvl - CVLPct::new(1),
            }
        }

        fn collateralization_update(
            last_state: CollateralizationState,
            cvl: CVLPct,
        ) -> Option<CollateralizationState> {
            cvl.collateralization_update(default_terms(), last_state, None, false)
        }

        fn collateralization_update_with_buffer(
            last_state: CollateralizationState,
            cvl: CVLPct,
        ) -> Option<CollateralizationState> {
            cvl.collateralization_update(
                default_terms(),
                last_state,
                Some(default_upgrade_buffer_cvl_pct()),
                false,
            )
        }

        fn collateralization_update_with_liquidation_limit(
            last_state: CollateralizationState,
            cvl: CVLPct,
        ) -> Option<CollateralizationState> {
            cvl.collateralization_update(default_terms(), last_state, None, true)
        }

        fn all_collaterization_update_fns(
        ) -> Vec<fn(CollateralizationState, CVLPct) -> Option<CollateralizationState>> {
            vec![
                collateralization_update,
                collateralization_update_with_buffer,
                collateralization_update_with_liquidation_limit,
            ]
        }

        #[test]
        fn fully_collateralized_to_fully_collateralized() {
            for collateralization_update in all_collaterization_update_fns() {
                assert_eq!(
                    collateralization_update(
                        CollateralizationState::FullyCollateralized,
                        cvl_test_values().above_fully_collateralized + CVLPct::new(1),
                    ),
                    None
                );
            }
        }

        #[test]
        fn fully_collateralized_to_under_margin_called() {
            for collateralization_update in all_collaterization_update_fns() {
                assert_eq!(
                    collateralization_update(
                        CollateralizationState::FullyCollateralized,
                        cvl_test_values().below_margin_called,
                    ),
                    Some(CollateralizationState::UnderMarginCallThreshold)
                );
            }
        }

        #[test]
        fn fully_collateralized_to_under_liquidation() {
            for collateralization_update in all_collaterization_update_fns() {
                assert_eq!(
                    collateralization_update(
                        CollateralizationState::FullyCollateralized,
                        cvl_test_values().below_liquidation,
                    ),
                    Some(CollateralizationState::UnderLiquidationThreshold)
                );
            }
        }

        #[test]
        fn under_margin_called_to_above_margin_called_and_below_buffer() {
            assert_eq!(
                collateralization_update(
                    CollateralizationState::UnderMarginCallThreshold,
                    cvl_test_values().above_margin_called_and_below_buffer,
                ),
                Some(CollateralizationState::FullyCollateralized)
            );

            assert_eq!(
                collateralization_update_with_buffer(
                    CollateralizationState::UnderMarginCallThreshold,
                    cvl_test_values().above_margin_called_and_below_buffer,
                ),
                None
            );

            assert_eq!(
                collateralization_update_with_liquidation_limit(
                    CollateralizationState::UnderMarginCallThreshold,
                    cvl_test_values().above_margin_called_and_below_buffer,
                ),
                Some(CollateralizationState::FullyCollateralized)
            );
        }

        #[test]
        fn under_margin_called_to_fully_collateralized() {
            for collateralization_update in all_collaterization_update_fns() {
                assert_eq!(
                    collateralization_update(
                        CollateralizationState::UnderMarginCallThreshold,
                        cvl_test_values().above_margin_called_and_buffer,
                    ),
                    Some(CollateralizationState::FullyCollateralized),
                );
            }
        }

        #[test]
        fn under_margin_called_to_under_liquidation() {
            for collateralization_update in all_collaterization_update_fns() {
                assert_eq!(
                    collateralization_update(
                        CollateralizationState::UnderMarginCallThreshold,
                        cvl_test_values().below_liquidation,
                    ),
                    Some(CollateralizationState::UnderLiquidationThreshold),
                );
            }
        }

        #[test]
        fn under_liquidation_to_fully_collateralized() {
            assert_eq!(
                collateralization_update(
                    CollateralizationState::UnderLiquidationThreshold,
                    cvl_test_values().above_fully_collateralized,
                ),
                Some(CollateralizationState::FullyCollateralized),
            );

            assert_eq!(
                collateralization_update_with_buffer(
                    CollateralizationState::UnderLiquidationThreshold,
                    cvl_test_values().above_fully_collateralized,
                ),
                Some(CollateralizationState::FullyCollateralized),
            );

            assert_eq!(
                collateralization_update_with_liquidation_limit(
                    CollateralizationState::UnderLiquidationThreshold,
                    cvl_test_values().above_fully_collateralized,
                ),
                None,
            );
        }
    }
}
