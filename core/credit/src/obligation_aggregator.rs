use crate::{
    obligation::{Obligation, ObligationType, ObligationsOutstanding},
    primitives::*,
    CoreCreditError,
};

pub struct ObligationAggregator {
    obligations: Vec<ObligationDataForAggregation>,
}

#[derive(Debug, Clone, Copy)]
pub struct ObligationDataForAggregation {
    obligation_type: ObligationType,
    outstanding: UsdCents,
}

impl From<&Obligation> for ObligationDataForAggregation {
    fn from(obligation: &Obligation) -> Self {
        Self {
            obligation_type: obligation.obligation_type(),
            outstanding: obligation.outstanding(),
        }
    }
}

impl ObligationAggregator {
    pub fn new(obligations: impl Into<Vec<ObligationDataForAggregation>>) -> Self {
        Self {
            obligations: obligations.into(),
        }
    }

    pub fn outstanding(&self) -> Result<ObligationsOutstanding, CoreCreditError> {
        let mut disbursal_obligations = vec![];
        let mut interest_obligations = vec![];
        for obligation in &self.obligations {
            match obligation.obligation_type {
                ObligationType::Disbursal => disbursal_obligations.push(obligation),
                ObligationType::Interest => interest_obligations.push(obligation),
            }
        }
        let disbursed_outstanding = disbursal_obligations
            .iter()
            .map(|o| o.outstanding)
            .fold(UsdCents::ZERO, |acc, amount| acc + amount);
        let interest_outstanding = interest_obligations
            .iter()
            .map(|o| o.outstanding)
            .fold(UsdCents::ZERO, |acc, amount| acc + amount);

        Ok(ObligationsOutstanding {
            disbursed: disbursed_outstanding,
            interest: interest_outstanding,
        })
    }
}

// TODO: Add Tests
