use crate::{payment_allocation::PaymentAllocation, primitives::*};

pub struct PaymentAllocations {
    pub amount_allocated: UsdCents,

    allocations: Vec<PaymentAllocation>,
}

impl IntoIterator for PaymentAllocations {
    type Item = PaymentAllocation;
    type IntoIter = std::vec::IntoIter<PaymentAllocation>;

    fn into_iter(self) -> Self::IntoIter {
        self.allocations.into_iter()
    }
}

impl PaymentAllocations {
    pub(super) fn new(allocations: Vec<PaymentAllocation>) -> Self {
        let amount_allocated = allocations.iter().fold(UsdCents::ZERO, |c, a| c + a.amount);
        Self {
            allocations,
            amount_allocated,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use audit::{AuditEntryId, AuditInfo};
    use es_entity::*;

    use crate::NewPaymentAllocation;

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn allocation(amount: UsdCents) -> PaymentAllocation {
        let new_allocation = NewPaymentAllocation::builder()
            .id(PaymentAllocationId::new())
            .payment_id(PaymentId::new())
            .credit_facility_id(CreditFacilityId::new())
            .obligation_id(ObligationId::new())
            .obligation_allocation_idx(0)
            .obligation_type(ObligationType::Disbursal)
            .receivable_account_id(CalaAccountId::new())
            .account_to_be_debited_id(CalaAccountId::new())
            .effective(Utc::now().date_naive())
            .amount(amount)
            .audit_info(dummy_audit_info())
            .build()
            .unwrap();

        PaymentAllocation::try_from_events(new_allocation.into_events()).unwrap()
    }

    #[test]
    fn amount_allocated_is_zero_for_empty_vec() {
        let allocations = PaymentAllocations::new(vec![]);
        assert_eq!(allocations.amount_allocated, UsdCents::ZERO);
    }

    #[test]
    fn amount_allocated_for_populated_vec() {
        let a1 = UsdCents::from(1);
        let a2 = UsdCents::from(2);
        let allocations = PaymentAllocations::new(vec![allocation(a1), allocation(a2)]);
        assert_eq!(allocations.amount_allocated, a1 + a2);
    }

    #[test]
    fn can_iterate_over_payment_allocations() {
        let a1 = UsdCents::from(1);
        let a2 = UsdCents::from(2);
        let a3 = UsdCents::from(3);
        let allocations_vec = vec![allocation(a1), allocation(a2), allocation(a3)];
        let allocation_ids = allocations_vec.iter().map(|a| a.id).collect::<Vec<_>>();
        let allocations = PaymentAllocations::new(allocations_vec);

        let items = allocations.into_iter().map(|a| a.id).collect::<Vec<_>>();
        assert_eq!(items, allocation_ids);
    }
}
