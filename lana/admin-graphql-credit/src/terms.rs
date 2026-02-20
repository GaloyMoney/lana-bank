use async_graphql::*;
use serde::{Deserialize, Serialize};

pub use admin_graphql_shared::credit::{
    CVLPct, Duration, FiniteCVLPct, InfiniteCVLPct, Period, TermValues,
};

pub use lana_app::terms::{
    AnnualRatePct, CVLPct as DomainCVLPct, DisbursalPolicy, FacilityDuration as DomainDuration,
    InterestInterval, ObligationDuration as DomainObligationDuration, OneTimeFeeRatePct,
    TermValues as DomainTermValues,
};

#[derive(InputObject)]
pub struct TermsInput {
    pub annual_rate: AnnualRatePct,
    pub accrual_interval: InterestInterval,
    pub accrual_cycle_interval: InterestInterval,
    pub one_time_fee_rate: OneTimeFeeRatePct,
    pub disbursal_policy: DisbursalPolicy,
    pub duration: DurationInput,
    pub interest_due_duration_from_accrual: DurationInput,
    pub obligation_overdue_duration_from_due: DurationInput,
    pub obligation_liquidation_duration_from_due: DurationInput,
    pub margin_call_cvl: CVLPctValue,
    pub initial_cvl: CVLPctValue,
    pub liquidation_cvl: CVLPctValue,
}

#[derive(InputObject)]
pub struct DurationInput {
    pub period: Period,
    pub units: u32,
}

impl From<DurationInput> for DomainDuration {
    fn from(duration: DurationInput) -> Self {
        match duration.period {
            Period::Months => Self::Months(duration.units),
            Period::Days => todo!(),
        }
    }
}

impl From<DurationInput> for DomainObligationDuration {
    fn from(duration: DurationInput) -> Self {
        match duration.period {
            Period::Months => todo!(),
            Period::Days => Self::Days(duration.units.into()),
        }
    }
}

impl From<DurationInput> for Option<DomainObligationDuration> {
    fn from(duration: DurationInput) -> Self {
        match duration.period {
            Period::Months => todo!(),
            Period::Days => Some(DomainObligationDuration::Days(duration.units.into())),
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
