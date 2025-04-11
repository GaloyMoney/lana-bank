use crate::primitives::CreditFacilityId;

use super::{entity::Obligation, payment_allocator::ObligationDataForAllocation};

#[allow(dead_code)]
pub struct FacilityObligations {
    facility_id: CreditFacilityId,
    obligations: Vec<Obligation>,
}

impl IntoIterator for FacilityObligations {
    type Item = Obligation;
    type IntoIter = std::vec::IntoIter<Obligation>;

    fn into_iter(self) -> Self::IntoIter {
        self.obligations.into_iter()
    }
}

impl FacilityObligations {
    pub(super) fn new(facility_id: CreditFacilityId, obligations: Vec<Obligation>) -> Self {
        Self {
            facility_id,
            obligations,
        }
    }

    pub(super) fn data_for_allocation(&self) -> Vec<ObligationDataForAllocation> {
        self.obligations
            .iter()
            .map(ObligationDataForAllocation::from)
            .collect::<Vec<_>>()
    }
}
