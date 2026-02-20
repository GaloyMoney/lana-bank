use async_graphql::*;
use serde::{Deserialize, Serialize};

pub use lana_app::terms::{
    AnnualRatePct, CVLPct as DomainCVLPct, DisbursalPolicy, FacilityDuration as DomainDuration,
    InterestInterval, ObligationDuration as DomainObligationDuration, OneTimeFeeRatePct,
    TermValues as DomainTermValues,
};

#[derive(SimpleObject, Clone)]
pub struct TermValues {
    annual_rate: AnnualRatePct,
    accrual_interval: InterestInterval,
    accrual_cycle_interval: InterestInterval,
    one_time_fee_rate: OneTimeFeeRatePct,
    disbursal_policy: DisbursalPolicy,
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
            disbursal_policy: values.disbursal_policy,
            duration: values.duration.into(),
            liquidation_cvl: values.liquidation_cvl.into(),
            margin_call_cvl: values.margin_call_cvl.into(),
            initial_cvl: values.initial_cvl.into(),
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Period {
    Months,
    Days,
}

#[derive(SimpleObject, Clone)]
pub struct Duration {
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

impl From<DomainObligationDuration> for Duration {
    fn from(duration: DomainObligationDuration) -> Self {
        match duration {
            DomainObligationDuration::Days(days) => Self {
                period: Period::Days,
                units: days.try_into().expect("Days number too large"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CVLPctValue(rust_decimal::Decimal);
async_graphql::scalar!(CVLPctValue);

impl From<CVLPctValue> for DomainCVLPct {
    fn from(input: CVLPctValue) -> Self {
        DomainCVLPct::from(input.0)
    }
}

#[derive(async_graphql::Union, Clone)]
pub enum CVLPct {
    Finite(FiniteCVLPct),
    Infinite(InfiniteCVLPct),
}

#[derive(SimpleObject, Clone)]
pub struct FiniteCVLPct {
    value: CVLPctValue,
}

#[derive(SimpleObject, Clone)]
pub struct InfiniteCVLPct {
    is_infinite: bool,
}

impl From<DomainCVLPct> for CVLPct {
    fn from(cvl: DomainCVLPct) -> Self {
        match cvl {
            DomainCVLPct::Finite(value) => CVLPct::Finite(FiniteCVLPct {
                value: CVLPctValue(value),
            }),
            DomainCVLPct::Infinite => CVLPct::Infinite(InfiniteCVLPct { is_infinite: true }),
        }
    }
}
