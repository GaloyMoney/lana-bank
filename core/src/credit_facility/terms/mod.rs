pub mod error;
mod repo;
mod value;

use crate::primitives::CreditFacilityTermsId;

pub use repo::*;
pub use value::*;

pub struct Terms {
    pub id: CreditFacilityTermsId,
    pub values: TermValues,
}
