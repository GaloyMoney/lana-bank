use super::entity::Obligation;
use crate::primitives::CreditFacilityId;

#[allow(dead_code)]
pub struct FacilityObligations {
    facility_id: CreditFacilityId,
    obligations: Vec<Obligation>,
}

impl FacilityObligations {
    pub(super) fn new(facility_id: CreditFacilityId, obligations: Vec<Obligation>) -> Self {
        Self {
            facility_id,
            obligations,
        }
    }
}
