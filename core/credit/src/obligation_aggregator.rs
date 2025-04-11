use crate::{
    obligation::{Obligation, ObligationStatus, ObligationType, ObligationsOutstanding},
    primitives::*,
    ObligationsAmounts,
};

pub struct ObligationAggregator {
    obligations: Vec<ObligationDataForAggregation>,
}

#[derive(Debug, Clone, Copy)]
pub struct ObligationDataForAggregation {
    obligation_type: ObligationType,
    status: ObligationStatus,
    initial_amount: UsdCents,
    outstanding: UsdCents,
}

impl From<&Obligation> for ObligationDataForAggregation {
    fn from(obligation: &Obligation) -> Self {
        Self {
            obligation_type: obligation.obligation_type(),
            status: obligation.status(),
            initial_amount: obligation.initial_amount,
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

    pub fn has_confirmed_disbursals(&self) -> bool {
        self.obligations
            .iter()
            .any(|obligation| obligation.obligation_type == ObligationType::Disbursal)
    }

    pub fn initial_amounts(&self) -> ObligationsAmounts {
        let mut res = ObligationsAmounts::ZERO;
        for obligation in &self.obligations {
            match obligation.obligation_type {
                ObligationType::Disbursal => res.disbursed += obligation.initial_amount,
                ObligationType::Interest => res.interest += obligation.initial_amount,
            }
        }

        res
    }

    pub fn outstanding(&self) -> ObligationsOutstanding {
        let mut res = ObligationsOutstanding::ZERO;
        for obligation in &self.obligations {
            match obligation.status {
                ObligationStatus::NotYetDue => match obligation.obligation_type {
                    ObligationType::Disbursal => {
                        res.not_yet_due.disbursed += obligation.outstanding
                    }
                    ObligationType::Interest => res.not_yet_due.interest += obligation.outstanding,
                },
                ObligationStatus::Due => match obligation.obligation_type {
                    ObligationType::Disbursal => res.due.disbursed += obligation.outstanding,
                    ObligationType::Interest => res.due.interest += obligation.outstanding,
                },
                ObligationStatus::Overdue => match obligation.obligation_type {
                    ObligationType::Disbursal => res.overdue.disbursed += obligation.outstanding,
                    ObligationType::Interest => res.overdue.interest += obligation.outstanding,
                },
                ObligationStatus::Defaulted => match obligation.obligation_type {
                    ObligationType::Disbursal => res.defaulted.disbursed += obligation.outstanding,
                    ObligationType::Interest => res.defaulted.interest += obligation.outstanding,
                },
                ObligationStatus::Paid => (),
            }
        }

        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_aggregate_outstanding() {
        let obligations = vec![
            ObligationDataForAggregation {
                obligation_type: ObligationType::Disbursal,
                status: ObligationStatus::NotYetDue,
                initial_amount: UsdCents::from(1),
                outstanding: UsdCents::from(1),
            },
            ObligationDataForAggregation {
                obligation_type: ObligationType::Interest,
                status: ObligationStatus::NotYetDue,
                initial_amount: UsdCents::from(2),
                outstanding: UsdCents::from(2),
            },
            ObligationDataForAggregation {
                obligation_type: ObligationType::Disbursal,
                status: ObligationStatus::Due,
                initial_amount: UsdCents::from(3),
                outstanding: UsdCents::from(3),
            },
            ObligationDataForAggregation {
                obligation_type: ObligationType::Interest,
                status: ObligationStatus::Due,
                initial_amount: UsdCents::from(4),
                outstanding: UsdCents::from(4),
            },
            ObligationDataForAggregation {
                obligation_type: ObligationType::Disbursal,
                status: ObligationStatus::Overdue,
                initial_amount: UsdCents::from(5),
                outstanding: UsdCents::from(5),
            },
            ObligationDataForAggregation {
                obligation_type: ObligationType::Interest,
                status: ObligationStatus::Overdue,
                initial_amount: UsdCents::from(6),
                outstanding: UsdCents::from(6),
            },
            ObligationDataForAggregation {
                obligation_type: ObligationType::Disbursal,
                status: ObligationStatus::Defaulted,
                initial_amount: UsdCents::from(7),
                outstanding: UsdCents::from(7),
            },
            ObligationDataForAggregation {
                obligation_type: ObligationType::Interest,
                status: ObligationStatus::Defaulted,
                initial_amount: UsdCents::from(8),
                outstanding: UsdCents::from(8),
            },
        ];

        let res = ObligationAggregator::new(obligations).outstanding();
        assert_eq!(res.not_yet_due.disbursed, UsdCents::from(1));
        assert_eq!(res.not_yet_due.interest, UsdCents::from(2));
        assert_eq!(res.due.disbursed, UsdCents::from(3));
        assert_eq!(res.due.interest, UsdCents::from(4));
        assert_eq!(res.overdue.disbursed, UsdCents::from(5));
        assert_eq!(res.overdue.interest, UsdCents::from(6));
        assert_eq!(res.defaulted.disbursed, UsdCents::from(7));
        assert_eq!(res.defaulted.interest, UsdCents::from(8));
    }
}
