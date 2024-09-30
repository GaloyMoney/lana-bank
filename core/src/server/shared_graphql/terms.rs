use async_graphql::*;

use crate::server::shared_graphql::{convert::*, primitives::UUID};

pub use crate::loan::AnnualRatePct;
scalar!(AnnualRatePct);
pub use crate::loan::CVLPct;
scalar!(CVLPct);

#[derive(SimpleObject)]
pub struct Terms {
    id: ID,
    terms_id: UUID,
    values: TermValues,
}

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
#[graphql(remote = "crate::loan::InterestInterval")]
pub enum InterestInterval {
    EndOfMonth,
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

impl ToGlobalId for crate::primitives::LoanTermsId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("loan_terms:{}", self))
    }
}

impl From<crate::loan::TermValues> for TermValues {
    fn from(values: crate::loan::TermValues) -> Self {
        Self {
            annual_rate: values.annual_rate,
            interval: values.interval.into(),
            duration: values.duration.into(),
            liquidation_cvl: values.liquidation_cvl,
            margin_call_cvl: values.margin_call_cvl,
            initial_cvl: values.initial_cvl,
        }
    }
}

impl From<crate::loan::Duration> for Duration {
    fn from(duration: crate::loan::Duration) -> Self {
        match duration {
            crate::loan::Duration::Months(months) => Self {
                period: Period::Months,
                units: months,
            },
        }
    }
}

impl From<DurationInput> for crate::loan::Duration {
    fn from(loan_duration: DurationInput) -> Self {
        match loan_duration.period {
            Period::Months => Self::Months(loan_duration.units),
        }
    }
}
