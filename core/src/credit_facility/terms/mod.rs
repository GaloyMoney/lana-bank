mod value;

use crate::primitives::CreditFacilityTermsId;

pub use value::*;

pub struct CreditFacilityTerms {
    pub id: CreditFacilityTermsId,
    pub values: CreditFacilityTermValues,
}
