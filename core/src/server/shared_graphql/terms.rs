use async_graphql::*;

pub use crate::terms::{AnnualRatePct, CVLPct, InterestInterval};

scalar!(AnnualRatePct);
scalar!(CVLPct);

#[derive(SimpleObject)]
pub struct TermValues {
    annual_rate: AnnualRatePct,
    interval: InterestInterval,
    duration: Duration,
    liquidation_cvl: CVLPct,
    margin_call_cvl: CVLPct,
    initial_cvl: CVLPct,
}

#[derive(SimpleObject)]
pub(super) struct Duration {
    period: Period,
    units: u32,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Period {
    Months,
}

#[derive(InputObject)]
pub struct DurationInput {
    pub period: Period,
    pub units: u32,
}

impl From<crate::terms::TermValues> for TermValues {
    fn from(values: crate::terms::TermValues) -> Self {
        Self {
            annual_rate: values.annual_rate,
            interval: values.interval,
            duration: values.duration.into(),
            liquidation_cvl: values.liquidation_cvl,
            margin_call_cvl: values.margin_call_cvl,
            initial_cvl: values.initial_cvl,
        }
    }
}

impl From<crate::terms::Duration> for Duration {
    fn from(duration: crate::terms::Duration) -> Self {
        match duration {
            crate::terms::Duration::Months(months) => Self {
                period: Period::Months,
                units: months,
            },
        }
    }
}

impl From<DurationInput> for crate::terms::Duration {
    fn from(loan_duration: DurationInput) -> Self {
        match loan_duration.period {
            Period::Months => Self::Months(loan_duration.units),
        }
    }
}
