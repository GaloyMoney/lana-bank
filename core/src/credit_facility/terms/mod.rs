mod value;

use crate::primitives::CreditFacilityTermsId;

pub use value::*;

pub struct Terms {
    pub id: CreditFacilityTermsId,
    pub values: TermValues,
}
