use chrono::{DateTime, Datelike, TimeZone, Utc};
use derive_builder::Builder;
use rust_decimal::{prelude::*, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::primitives::{CreditFacilityStatus, PriceOfOneBTC, Satoshis, UsdCents};

const NUMBER_OF_DAYS_IN_YEAR: Decimal = dec!(366);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, async_graphql::Enum)]
pub enum CollaterizationState {
    FullyCollateralized,
    UnderMarginCallThreshold,
    UnderLiquidationThreshold,
    NoCollateral,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct AnnualRatePct(Decimal);

impl AnnualRatePct {
    pub fn interest_for_time_period(&self, principal: UsdCents, days: u32) -> UsdCents {
        let cents = principal.to_usd() * Decimal::from(days) * self.0 / NUMBER_OF_DAYS_IN_YEAR;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CVLPct(Decimal);

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
        let ratio = (collateral_value.to_usd() / total_outstanding_amount.to_usd())
            .round_dp_with_strategy(2, RoundingStrategy::ToZero)
            * dec!(100);

        CVLPct::from(ratio)
    }

    pub fn is_significantly_lower_than(&self, other: CVLPct, buffer: CVLPct) -> bool {
        other > *self + buffer
    }

    pub fn collateralization(self, terms: TermValues) -> CollaterizationState {
        let margin_call_cvl = terms.margin_call_cvl;
        let liquidation_cvl = terms.liquidation_cvl;

        if self == CVLPct::ZERO {
            CollaterizationState::NoCollateral
        } else if self >= margin_call_cvl {
            CollaterizationState::FullyCollateralized
        } else if self >= liquidation_cvl {
            CollaterizationState::UnderMarginCallThreshold
        } else {
            CollaterizationState::UnderLiquidationThreshold
        }
    }

    pub fn collateralization_update(
        self,
        terms: TermValues,
        upgrade_buffer_cvl_pct: CVLPct,
        last_collateralization_state: CollaterizationState,
        current_status: CreditFacilityStatus,
    ) -> Option<CollaterizationState> {
        let calculated_collateralization = &self.collateralization(terms);

        match (
            current_status,
            last_collateralization_state,
            *calculated_collateralization,
        ) {
            // Redundant same state changes
            (_, CollaterizationState::NoCollateral, CollaterizationState::NoCollateral)
            | (
                _,
                CollaterizationState::FullyCollateralized,
                CollaterizationState::FullyCollateralized,
            )
            | (
                _,
                CollaterizationState::UnderMarginCallThreshold,
                CollaterizationState::UnderMarginCallThreshold,
            )
            | (
                _,
                CollaterizationState::UnderLiquidationThreshold,
                CollaterizationState::UnderLiquidationThreshold,
            ) => None,

            // Invalid collateral liquidation changes
            (CreditFacilityStatus::Active, CollaterizationState::UnderLiquidationThreshold, _) => {
                None
            }

            // Valid buffered collateral upgraded change
            (
                CreditFacilityStatus::Active,
                CollaterizationState::UnderMarginCallThreshold,
                CollaterizationState::FullyCollateralized,
            ) => {
                if terms
                    .margin_call_cvl
                    .is_significantly_lower_than(self, upgrade_buffer_cvl_pct)
                {
                    Some(*calculated_collateralization)
                } else {
                    None
                }
            }

            // Valid other collateral changes
            (_, CollaterizationState::NoCollateral, _)
            | (_, CollaterizationState::FullyCollateralized, _)
            | (_, CollaterizationState::UnderMarginCallThreshold, _)
            | (_, CollaterizationState::UnderLiquidationThreshold, _) => {
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
        Self::new(self.interval, self.end + chrono::Duration::days(1))
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
        }
    }
}

#[derive(Builder, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TermValues {
    #[builder(setter(into))]
    pub(crate) annual_rate: AnnualRatePct,
    #[builder(setter(into))]
    pub(crate) duration: Duration,
    #[builder(setter(into))]
    pub(crate) interval: InterestInterval,
    // overdue_penalty_rate: LoanAnnualRate,
    #[builder(setter(into))]
    pub(crate) liquidation_cvl: CVLPct,
    #[builder(setter(into))]
    pub(crate) margin_call_cvl: CVLPct,
    #[builder(setter(into))]
    pub(crate) initial_cvl: CVLPct,
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
    fn cvl_is_significantly_higher() {
        let buffer = CVLPct::new(5);

        let collateral_value = UsdCents::from(125000);
        let outstanding_amount = UsdCents::from(100000);
        let cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);

        let collateral_value = UsdCents::from(130999);
        let outstanding_amount = UsdCents::from(100000);
        let slightly_higher_cvl = CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(
            false,
            cvl.is_significantly_lower_than(slightly_higher_cvl, buffer)
        );

        let collateral_value = UsdCents::from(131000);
        let outstanding_amount = UsdCents::from(100000);
        let significantly_higher_cvl =
            CVLPct::from_loan_amounts(collateral_value, outstanding_amount);
        assert_eq!(
            true,
            cvl.is_significantly_lower_than(significantly_higher_cvl, buffer)
        );
    }

    fn terms() -> TermValues {
        TermValues::builder()
            .annual_rate(AnnualRatePct(dec!(12)))
            .duration(Duration::Months(3))
            .interval(InterestInterval::EndOfMonth)
            .liquidation_cvl(CVLPct(dec!(105)))
            .margin_call_cvl(CVLPct(dec!(125)))
            .initial_cvl(CVLPct(dec!(140)))
            .build()
            .expect("should build a valid term")
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
}
