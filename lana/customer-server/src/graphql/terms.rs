use async_graphql::*;

pub(super) use lana_app::terms::{
    AnnualRatePct, CVLPct, FacilityDuration as DomainDuration, InterestInterval, OneTimeFeeRatePct,
    TermValues as DomainTermValues,
};

#[derive(SimpleObject, Clone)]
pub(super) struct TermValues {
    annual_rate: AnnualRatePct,
    accrual_interval: InterestInterval,
    accrual_cycle_interval: InterestInterval,
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
            accrual_cycle_interval: values.accrual_cycle_interval,
            one_time_fee_rate: values.one_time_fee_rate,
            duration: values.duration.into(),
            liquidation_cvl: values.liquidation_cvl,
            margin_call_cvl: values.margin_call_cvl,
            initial_cvl: values.initial_cvl,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Period {
    Months,
}

#[derive(SimpleObject, Clone)]
pub(super) struct Duration {
    period: Period,
    units: u32,
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
