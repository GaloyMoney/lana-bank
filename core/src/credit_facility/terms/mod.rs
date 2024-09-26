pub mod error;
mod repo;
mod value;

use crate::primitives::CreditFacilityTermsId;

use error::*;
pub use value::*;

pub struct Terms {
    pub id: CreditFacilityTermsId,
    pub values: TermValues,
}
