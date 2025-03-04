use async_graphql::*;

pub use lana_app::terms::{
    AnnualRatePct, CVLPct, Duration as DomainDuration, InterestDuration as DomainInterestDuration,
    InterestInterval, OneTimeFeeRatePct, TermValues as DomainTermValues,
};

#[derive(SimpleObject, Clone)]
pub struct TermValues {
    annual_rate: AnnualRatePct,
    accrual_interval: InterestInterval,
    incurrence_interval: InterestInterval,
    one_time_fee_rate: OneTimeFeeRatePct,
    duration: Duration,
    liquidation_cvl: CVLPct,
    margin_call_cvl: CVLPct,
    initial_cvl: CVLPct,
}

impl From<DomainTermValues> for TermValues {
    fn from(values: DomainTermValues) -> Self {
        Self {
            annual_rate: values.annual_rate,
            accrual_interval: values.accrual_interval,
            incurrence_interval: values.incurrence_interval,
            one_time_fee_rate: values.one_time_fee_rate,
            duration: values.duration.into(),
            liquidation_cvl: values.liquidation_cvl,
            margin_call_cvl: values.margin_call_cvl,
            initial_cvl: values.initial_cvl,
        }
    }
}

#[derive(InputObject)]
pub struct TermsInput {
    pub annual_rate: AnnualRatePct,
    pub accrual_interval: InterestInterval,
    pub incurrence_interval: InterestInterval,
    pub liquidation_cvl: CVLPct,
    pub one_time_fee_rate: OneTimeFeeRatePct,
    pub duration: DurationInput,
    pub interest_due_duration: DurationInput,
    pub margin_call_cvl: CVLPct,
    pub initial_cvl: CVLPct,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Period {
    Months,
    Days,
}

#[derive(SimpleObject, Clone)]
pub(super) struct Duration {
    period: Period,
    units: u32,
}

#[derive(InputObject)]
pub struct DurationInput {
    pub period: Period,
    pub units: u32,
}

impl From<DomainDuration> for Duration {
    fn from(duration: DomainDuration) -> Self {
        match duration {
            DomainDuration::Months(months) => Self {
                period: Period::Months,
                units: months,
            },
        }
    }
}

impl From<DurationInput> for DomainDuration {
    fn from(duration: DurationInput) -> Self {
        match duration.period {
            Period::Months => Self::Months(duration.units),
            Period::Days => todo!(),
        }
    }
}

impl From<DomainInterestDuration> for Duration {
    fn from(duration: DomainInterestDuration) -> Self {
        match duration {
            DomainInterestDuration::Days(days) => Self {
                period: Period::Days,
                units: days.try_into().expect("Days number too large"),
            },
        }
    }
}

impl From<DurationInput> for DomainInterestDuration {
    fn from(duration: DurationInput) -> Self {
        match duration.period {
            Period::Months => todo!(),
            Period::Days => Self::Days(duration.units.into()),
        }
    }
}
