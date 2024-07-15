use chrono::{DateTime, Datelike, TimeZone, Utc};
use derive_builder::Builder;
use rust_decimal::{prelude::*, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::primitives::{PriceOfOneBTC, Satoshis, UsdCents};

const NUMBER_OF_DAYS_IN_YEAR: Decimal = dec!(366);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoanAnnualRate(Decimal);

impl LoanAnnualRate {
    fn interest_for_time_period(&self, principal: UsdCents, days: u32) -> UsdCents {
        let cents =
            principal.to_usd() * Decimal::from(days) * dec!(100) * self.0 / NUMBER_OF_DAYS_IN_YEAR;

        UsdCents::from(
            cents
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }
}

impl From<Decimal> for LoanAnnualRate {
    fn from(value: Decimal) -> Self {
        LoanAnnualRate(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoanPenaltyRatePct(Decimal);

impl LoanPenaltyRatePct {
    fn penalty(&self, principal: UsdCents) -> UsdCents {
        let cents = principal.to_usd() * self.0; // principal.to_usd() * penalty_rate_pct * dec!(100) / dec!(100);
        UsdCents::from(
            cents
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }
}

impl From<Decimal> for LoanPenaltyRatePct {
    fn from(value: Decimal) -> Self {
        LoanPenaltyRatePct(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoanCVLPct(Decimal);

impl LoanCVLPct {
    pub fn scale(&self, value: UsdCents) -> UsdCents {
        let cents = value.to_usd() * dec!(100) * (self.0 / dec!(100));
        UsdCents::from(
            cents
                .round_dp_with_strategy(0, RoundingStrategy::AwayFromZero)
                .to_u64()
                .expect("should return a valid integer"),
        )
    }
}

impl From<Decimal> for LoanCVLPct {
    fn from(value: Decimal) -> Self {
        LoanCVLPct(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum LoanDuration {
    Months(u32),
}

impl LoanDuration {
    pub fn expiration_date(&self, start_date: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            LoanDuration::Months(months) => start_date
                .checked_add_months(chrono::Months::new(*months))
                .expect("should return a expiration date"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterestInterval {
    EndOfMonth,
}

impl InterestInterval {
    pub fn next_interest_payment(&self, current_date: DateTime<Utc>) -> DateTime<Utc> {
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

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
pub struct TermValues {
    #[builder(setter(into))]
    pub(crate) annual_rate: LoanAnnualRate,
    #[builder(setter(into))]
    pub(crate) duration: LoanDuration,
    #[builder(setter(into))]
    pub(crate) interval: InterestInterval,
    #[builder(setter(into))]
    pub(crate) overdue_penalty: LoanPenaltyRatePct,
    #[builder(setter(into))]
    pub(crate) liquidation_cvl: LoanCVLPct,
    #[builder(setter(into))]
    pub(crate) margin_call_cvl: LoanCVLPct,
    #[builder(setter(into))]
    pub(crate) initial_cvl: LoanCVLPct,
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
        price.cents_to_sats(
            collateral_value,
            rust_decimal::RoundingStrategy::AwayFromZero,
        )
    }

    pub fn calculate_interest(&self, principal: UsdCents, days: u32) -> UsdCents {
        self.annual_rate.interest_for_time_period(principal, days)
    }

    pub fn calculate_penalty(&self, principal: UsdCents) -> UsdCents {
        self.overdue_penalty.penalty(principal)
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn loan_cvl_pct_scale() {
        let cvl = LoanCVLPct(dec!(140));
        let value = UsdCents::from(100000);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::from_usd(dec!(1400)));

        let cvl = LoanCVLPct(dec!(50));
        let value = UsdCents::from(333333);
        let scaled = cvl.scale(value);
        assert_eq!(scaled, UsdCents::from_usd(dec!(1666.67)));
    }

    fn terms() -> TermValues {
        TermValues::builder()
            .annual_rate(LoanAnnualRate(dec!(0.12)))
            .duration(LoanDuration::Months(3))
            .interval(InterestInterval::EndOfMonth)
            .overdue_penalty(LoanPenaltyRatePct(dec!(2)))
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    #[test]
    fn required_collateral() {
        let price = PriceOfOneBTC::new(UsdCents::from_usd(dec!(1000)));
        let terms = terms();
        let principal = UsdCents::from(100000);
        let required_collateral = terms.required_collateral(principal, price);
        let sats = Satoshis::from_btc(dec!(1.4));
        assert_eq!(required_collateral, sats);
    }

    #[test]
    fn next_interest_payment() {
        let interval = InterestInterval::EndOfMonth;
        let current_date = "2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let next_payment = "2024-12-31T23:59:59Z".parse::<DateTime<Utc>>().unwrap();

        assert_eq!(interval.next_interest_payment(current_date), next_payment);
    }

    #[test]
    fn interest_calculation() {
        let terms = terms();
        let principal = UsdCents::from_usd(dec!(100));
        let days = 366;
        let interest = terms.calculate_interest(principal, days);
        assert_eq!(interest, UsdCents::from(1200));

        let principal = UsdCents::from_usd(dec!(1000));
        let days = 23;
        let interest = terms.calculate_interest(principal, days);
        assert_eq!(interest, UsdCents::from(755));
    }

    #[test]
    fn penalty_calculation() {
        let terms = terms();
        let principal = UsdCents::from_usd(dec!(1));
        let penalty = terms.calculate_penalty(principal);
        assert_eq!(penalty, UsdCents::from(2));

        let principal = UsdCents::from_usd(dec!(1.21));
        let penalty = terms.calculate_penalty(principal);
        assert_eq!(penalty, UsdCents::from(3));
    }

    #[test]
    fn expiration_date() {
        let start_date = "2024-12-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let duration = LoanDuration::Months(3);
        let expiration_date = "2025-03-03T14:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(duration.expiration_date(start_date), expiration_date);
    }
}
