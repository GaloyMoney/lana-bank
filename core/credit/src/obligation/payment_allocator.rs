use chrono::{DateTime, Utc};

use crate::primitives::*;

use super::{
    entity::{Obligation, ObligationType},
    error::*,
};

pub struct PaymentAllocator {
    payment_id: PaymentId,
    amount: UsdCents,
}

#[derive(Clone)]
pub struct ObligationDataForAllocation {
    id: ObligationId,
    obligation_type: ObligationType,
    recorded_at: DateTime<Utc>,
    outstanding: UsdCents,
    receivable_account_id: CalaAccountId,
    account_to_be_debited_id: CalaAccountId,
}

impl From<&Obligation> for ObligationDataForAllocation {
    fn from(obligation: &Obligation) -> Self {
        Self {
            id: obligation.id,
            obligation_type: obligation.obligation_type(),
            recorded_at: obligation.recorded_at,
            outstanding: obligation.outstanding(),
            receivable_account_id: obligation
                .account_to_be_credited_id()
                .expect("Obligation was already paid"),
            account_to_be_debited_id: obligation
                .account_to_be_debited_id()
                .expect("Obligation was already paid"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NewPaymentAllocation {
    pub id: LedgerTxId,    // TODO: change to PaymentAllocationId
    pub tx_id: LedgerTxId, // TODO: change to PaymentAllocationId
    pub payment_id: PaymentId,
    pub obligation_id: ObligationId,
    pub obligation_type: ObligationType,
    pub receivable_account_id: CalaAccountId,
    pub account_to_be_debited_id: CalaAccountId,
    pub amount: UsdCents,
    pub recorded_at: DateTime<Utc>,
}

impl PaymentAllocator {
    pub fn new(payment_id: PaymentId, amount: UsdCents) -> Self {
        Self { payment_id, amount }
    }

    pub fn allocate(
        &self,
        obligations: impl Iterator<Item = impl Into<ObligationDataForAllocation>> + Clone,
    ) -> Result<Vec<NewPaymentAllocation>, ObligationError> {
        let outstanding = obligations
            .clone()
            .map(|o| {
                let data = o.into();
                data.outstanding
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount);
        if self.amount > outstanding {
            return Err(ObligationError::PaymentAmountGreaterThanOutstandingObligations);
        }

        let mut disbursal_obligations = vec![];
        let mut interest_obligations = vec![];
        for obligation in obligations {
            let data = obligation.into();
            match data.obligation_type {
                ObligationType::Disbursal => disbursal_obligations.push(data),
                ObligationType::Interest => interest_obligations.push(data),
            }
        }
        disbursal_obligations.sort_by_key(|obligation| obligation.recorded_at);
        interest_obligations.sort_by_key(|obligation| obligation.recorded_at);

        let mut sorted_obligations = vec![];
        sorted_obligations.extend(interest_obligations);
        sorted_obligations.extend(disbursal_obligations);

        let now = crate::time::now();
        let mut remaining = self.amount;
        let mut new_payment_allocations = vec![];
        for obligation in sorted_obligations {
            let payment_amount = std::cmp::min(remaining, obligation.outstanding);
            remaining -= payment_amount;

            let id = LedgerTxId::new();
            new_payment_allocations.push(NewPaymentAllocation {
                id,
                tx_id: id,
                payment_id: self.payment_id,
                obligation_id: obligation.id,
                obligation_type: obligation.obligation_type,
                receivable_account_id: obligation.receivable_account_id,
                account_to_be_debited_id: obligation.account_to_be_debited_id,
                amount: payment_amount,
                recorded_at: now,
            });

            if remaining == UsdCents::ZERO {
                break;
            }
        }

        Ok(new_payment_allocations)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_allocate_interest() {
        let allocator = PaymentAllocator::new(PaymentId::new(), UsdCents::ONE);

        let obligation_type = ObligationType::Interest;
        let obligations = vec![ObligationDataForAllocation {
            id: ObligationId::new(),
            obligation_type,
            recorded_at: Utc::now(),
            outstanding: UsdCents::ONE,
            receivable_account_id: CalaAccountId::new(),
            account_to_be_debited_id: CalaAccountId::new(),
        }];

        let new_allocations = allocator.allocate(obligations.into_iter()).unwrap();
        assert_eq!(new_allocations.len(), 1);
    }

    #[test]
    fn can_allocate_disbursal() {
        let allocator = PaymentAllocator::new(PaymentId::new(), UsdCents::ONE);

        let obligation_type = ObligationType::Disbursal;
        let obligations = vec![ObligationDataForAllocation {
            id: ObligationId::new(),
            obligation_type,
            recorded_at: Utc::now(),
            outstanding: UsdCents::ONE,
            receivable_account_id: CalaAccountId::new(),
            account_to_be_debited_id: CalaAccountId::new(),
        }];

        let new_allocations = allocator.allocate(obligations.into_iter()).unwrap();
        assert_eq!(new_allocations.len(), 1);
    }

    #[test]
    fn can_allocate_interest_and_disbursal() {
        let allocator = PaymentAllocator::new(PaymentId::new(), UsdCents::from(2));

        let obligations = vec![
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                recorded_at: Utc::now(),
                outstanding: UsdCents::ONE,
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                recorded_at: Utc::now(),
                outstanding: UsdCents::ONE,
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
        ];

        let new_allocations = allocator.allocate(obligations.into_iter()).unwrap();
        assert_eq!(new_allocations.len(), 2);
    }

    #[test]
    fn can_allocate_partially() {
        let allocator = PaymentAllocator::new(PaymentId::new(), UsdCents::from(5));

        let obligations = vec![
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                recorded_at: Utc::now(),
                outstanding: UsdCents::from(4),
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                recorded_at: Utc::now(),
                outstanding: UsdCents::from(3),
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
        ];

        let new_allocations = allocator.allocate(obligations.into_iter()).unwrap();

        assert_eq!(new_allocations[0].amount, UsdCents::from(4));
        assert_eq!(new_allocations[1].amount, UsdCents::from(1));
    }

    #[test]
    fn errors_if_greater_than_outstanding() {
        let allocator = PaymentAllocator::new(PaymentId::new(), UsdCents::from(3));

        let obligations = vec![
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                recorded_at: Utc::now(),
                outstanding: UsdCents::ONE,
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                recorded_at: Utc::now(),
                outstanding: UsdCents::ONE,
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
        ];

        assert!(allocator.allocate(obligations.into_iter()).is_err());
    }

    #[test]
    fn allocates_interest_first() {
        let allocator = PaymentAllocator::new(PaymentId::new(), UsdCents::from(10));

        let obligations = vec![
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                recorded_at: Utc::now(),
                outstanding: UsdCents::from(2),
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                recorded_at: Utc::now(),
                outstanding: UsdCents::from(4),
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Interest,
                recorded_at: Utc::now(),
                outstanding: UsdCents::from(3),
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
            ObligationDataForAllocation {
                id: ObligationId::new(),
                obligation_type: ObligationType::Disbursal,
                recorded_at: Utc::now(),
                outstanding: UsdCents::from(1),
                receivable_account_id: CalaAccountId::new(),
                account_to_be_debited_id: CalaAccountId::new(),
            },
        ];

        let new_allocations = allocator.allocate(obligations.into_iter()).unwrap();
        assert_eq!(new_allocations.len(), 4);

        assert_eq!(new_allocations[0].amount, UsdCents::from(4));
        assert_eq!(new_allocations[1].amount, UsdCents::from(3));

        assert_eq!(new_allocations[2].amount, UsdCents::from(2));
        assert_eq!(new_allocations[3].amount, UsdCents::from(1));
    }
}
