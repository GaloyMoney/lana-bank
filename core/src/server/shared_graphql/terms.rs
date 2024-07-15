use async_graphql::*;

use crate::{
    loan::{LoanAnnualRate, LoanCVLPct},
    server::shared_graphql::{convert::*, primitives::UUID},
};

scalar!(LoanAnnualRate);
scalar!(LoanCVLPct);

#[derive(SimpleObject)]
pub struct Terms {
    id: ID,
    terms_id: UUID,
    values: TermValues,
}

#[derive(SimpleObject)]
pub struct TermValues {
    annual_rate: LoanAnnualRate,
    interval: InterestInterval,
    duration: LoanDuration,
    liquidation_cvl: LoanCVLPct,
    margin_call_cvl: LoanCVLPct,
    initial_cvl: LoanCVLPct,
}

#[derive(SimpleObject)]
pub(super) struct LoanDuration {
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

impl ToGlobalId for crate::primitives::LoanTermsId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("loan_terms:{}", self))
    }
}

impl From<crate::loan::Terms> for Terms {
    fn from(terms: crate::loan::Terms) -> Self {
        Self {
            id: terms.id.to_global_id(),
            terms_id: terms.id.into(),
            values: terms.values.into(),
        }
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

impl From<crate::loan::LoanDuration> for LoanDuration {
    fn from(duration: crate::loan::LoanDuration) -> Self {
        match duration {
            crate::loan::LoanDuration::Months(months) => Self {
                period: Period::Months,
                units: months,
            },
        }
    }
}
