use async_graphql::*;

use crate::{
    loan::{LoanAnnualRate, LoanCVLPct},
    server::shared_graphql::primitives::UUID,
};

use super::convert::ToGlobalId;

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
    r#type: DurationType,
    duration: u32,
}

#[derive(InputObject)]
pub(super) struct TermValuesCreateInput {
    pub annual_rate: LoanAnnualRate,
    pub interval: InterestInterval,
    pub liquidation_cvl: LoanCVLPct,
    pub duration: LoanDurationInput,
    pub margin_call_cvl: LoanCVLPct,
    pub initial_cvl: LoanCVLPct,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "crate::loan::InterestInterval")]
pub(super) enum InterestInterval {
    EndOfMonth,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]

pub(super) enum DurationType {
    Months,
}

#[derive(InputObject)]
pub(super) struct LoanDurationInput {
    pub r#type: DurationType,
    pub duration: u32,
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
                r#type: DurationType::Months,
                duration: months,
            },
        }
    }
}

impl From<LoanDurationInput> for crate::loan::LoanDuration {
    fn from(loan_duration: LoanDurationInput) -> Self {
        match loan_duration.r#type {
            DurationType::Months => Self::Months(loan_duration.duration),
        }
    }
}

#[derive(SimpleObject)]
pub struct TermValuesCreatePayload {
    pub terms: Terms,
}

impl From<crate::loan::Terms> for TermValuesCreatePayload {
    fn from(terms: crate::loan::Terms) -> Self {
        Self {
            terms: terms.into(),
        }
    }
}
