use chrono::{DateTime, Utc};

use crate::{obligation::Obligation, primitives::*, CoreCreditError};

pub struct PaymentAllocator {
    payment_id: PaymentId,
    amount: UsdCents,
}

pub struct ObligationDataForAllocation {
    id: ObligationId,
    recorded_at: DateTime<Utc>,
    outstanding: UsdCents,
    receivable_account_id: CalaAccountId,
    account_to_be_debited_id: CalaAccountId,
}

impl From<&Obligation> for ObligationDataForAllocation {
    fn from(obligation: &Obligation) -> Self {
        Self {
            id: obligation.id,
            recorded_at: obligation.recorded_at,
            outstanding: obligation.outstanding(),
            receivable_account_id: obligation.account_to_be_credited_id,
            account_to_be_debited_id: obligation.account_to_be_debited_id,
        }
    }
}

pub struct PaymentAllocation {
    pub id: LedgerTxId,    // TODO: change to PaymentAllocationId
    pub tx_id: LedgerTxId, // TODO: change to PaymentAllocationId
    pub payment_id: PaymentId,
    pub obligation_id: ObligationId,
    pub receivable_account_id: CalaAccountId,
    pub account_to_be_debited_id: CalaAccountId,
    pub amount: UsdCents,
}

impl PaymentAllocator {
    pub fn new(payment_id: PaymentId, amount: UsdCents) -> Self {
        Self { payment_id, amount }
    }

    pub fn allocate(
        &self,
        mut obligations: Vec<ObligationDataForAllocation>,
    ) -> Result<Vec<PaymentAllocation>, CoreCreditError> {
        let outstanding = obligations
            .iter()
            .map(|o| o.outstanding)
            .fold(UsdCents::ZERO, |acc, amount| acc + amount);
        if self.amount > outstanding {
            return Err(CoreCreditError::PaymentAmountGreaterThanOutstandingObligations);
        }

        obligations.sort_by_key(|obligation| obligation.recorded_at);
        let mut remaining = self.amount;
        let mut new_payment_allocations = vec![];
        for obligation in obligations {
            let payment_amount = std::cmp::min(remaining, obligation.outstanding);
            remaining -= payment_amount;

            let id = LedgerTxId::new();
            new_payment_allocations.push(PaymentAllocation {
                id,
                tx_id: id,
                payment_id: self.payment_id,
                obligation_id: obligation.id,
                receivable_account_id: obligation.receivable_account_id,
                account_to_be_debited_id: obligation.account_to_be_debited_id,
                amount: payment_amount,
            });
            if remaining == UsdCents::ZERO {
                break;
            }
        }

        Ok(new_payment_allocations)
    }
}
