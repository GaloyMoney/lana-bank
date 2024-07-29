use chrono::{DateTime, Datelike, TimeZone, Utc};
use derive_builder::Builder;
use rust_decimal::{prelude::*, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use super::error::*;

use crate::primitives::{PriceOfOneBTC, Satoshis, UsdCents};

const NUMBER_OF_DAYS_IN_YEAR: Decimal = dec!(366);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct AnnualRate(Decimal);

impl AnnualRate {
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

impl From<Decimal> for AnnualRate {
    fn from(value: Decimal) -> Self {
        AnnualRate(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CVLPct(Decimal);

impl CVLPct {
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
    pub(crate) annual_rate: AnnualRate,
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
    ) -> Result<Satoshis, LoanTermsError> {
        let collateral_value = self.initial_cvl.scale(desired_principal);
        Ok(price.try_cents_to_sats(
            collateral_value,
            rust_decimal::RoundingStrategy::AwayFromZero,
        )?)
    }

    pub fn calculate_interest(&self, principal: UsdCents, days: u32) -> UsdCents {
        self.annual_rate.interest_for_time_period(principal, days)
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

    fn terms() -> TermValues {
        TermValues::builder()
            .annual_rate(AnnualRate(dec!(0.12)))
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
        let required_collateral = terms.required_collateral(principal, price).unwrap();
        let sats = Satoshis::try_from_btc(dec!(1.4)).unwrap();
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
        let principal = UsdCents::try_from_usd(dec!(100)).unwrap();
        let days = 366;
        let interest = terms.calculate_interest(principal, days);
        assert_eq!(interest, UsdCents::from(1200));

        let principal = UsdCents::try_from_usd(dec!(1000)).unwrap();
        let days = 23;
        let interest = terms.calculate_interest(principal, days);
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
