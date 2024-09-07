use chrono::{DateTime, Datelike, TimeZone, Utc};
use derive_builder::Builder;
use rust_decimal::{prelude::*, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::primitives::{PriceOfOneBTC, Satoshis, UsdCents};

use super::error::*;

const NUMBER_OF_DAYS_IN_YEAR: Decimal = dec!(366);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
}

impl From<Decimal> for CVLPct {
    fn from(value: Decimal) -> Self {
        CVLPct(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct InterestPeriodStartDate(DateTime<Utc>);

impl PartialEq<DateTime<Utc>> for InterestPeriodStartDate {
    fn eq(&self, other: &DateTime<Utc>) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<DateTime<Utc>> for InterestPeriodStartDate {
    fn partial_cmp(&self, other: &DateTime<Utc>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl InterestPeriodStartDate {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn maybe_if_before_now(&self) -> Option<Self> {
        if *self < Utc::now() {
            Some(*self)
        } else {
            None
        }
    }

    pub fn absolute_end_date_for_period(
        &self,
        interval: InterestInterval,
    ) -> InterestPeriodEndDate {
        interval.end_date_for_period(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InterestPeriodEndDate(DateTime<Utc>);

impl From<InterestPeriodEndDate> for DateTime<Utc> {
    fn from(end_date: InterestPeriodEndDate) -> Self {
        end_date.0
    }
}

impl InterestPeriodEndDate {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn next_start_date(&self) -> InterestPeriodStartDate {
        InterestPeriodStartDate::new(self.0 + chrono::Duration::days(1))
    }

    pub fn days_in_period(
        &self,
        start_date: InterestPeriodStartDate,
    ) -> Result<u32, LoanTermsError> {
        if start_date.0 > self.0 {
            return Err(LoanTermsError::InvalidFutureDateComparisonForAccrualDate(
                self.0,
                start_date.0,
            ));
        }
        Ok(self.0.day() - start_date.0.day() + 1)
    }
}

pub struct InterestPeriod {
    pub start: InterestPeriodStartDate,
    pub end: InterestPeriodEndDate,
}

impl InterestPeriod {
    pub fn new(
        start_date: InterestPeriodStartDate,
        end_date: InterestPeriodEndDate,
    ) -> Result<Self, LoanTermsError> {
        if start_date.0 > end_date.0 {
            return Err(LoanTermsError::InvalidFutureDateComparisonForAccrualDate(
                end_date.0,
                start_date.0,
            ));
        }

        Ok(Self {
            start: start_date,
            end: end_date,
        })
    }

    pub fn days(&self) -> u32 {
        self.end
            .days_in_period(self.start)
            .expect("Impossible cmp error for struct")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterestInterval {
    EndOfMonth,
}

impl InterestInterval {
    pub fn end_date_for_period(&self, current_date: DateTime<Utc>) -> InterestPeriodEndDate {
        match self {
            InterestInterval::EndOfMonth => {
                let current_year = current_date.year();
                let current_month = current_date.month();

                let (year, month) = if current_month == 12 {
                    (current_year + 1, 1)
                } else {
                    (current_year, current_month + 1)
                };

                InterestPeriodEndDate(
                    Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0)
                        .single()
                        .expect("should return a valid date time")
                        - chrono::Duration::seconds(1),
                )
            }
        }
    }
}

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
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
    fn next_interest_accrual_date() {
        let interval = InterestInterval::EndOfMonth;
        let current_date =
            InterestPeriodStartDate::new("2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
        let next_payment =
            InterestPeriodEndDate("2024-12-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap());

        assert_eq!(
            current_date.absolute_end_date_for_period(interval),
            next_payment
        );
    }

    #[test]
    fn days_in_period() {
        let end_date =
            InterestPeriodEndDate::new("2024-12-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap());

        let start_date =
            InterestPeriodStartDate::new("2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(end_date.days_in_period(start_date).unwrap(), 29);

        let start_date =
            InterestPeriodStartDate::new("2024-12-01T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(end_date.days_in_period(start_date).unwrap(), 31);

        let start_date =
            InterestPeriodStartDate::new("2025-01-01T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert!(end_date.days_in_period(start_date).is_err());
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
