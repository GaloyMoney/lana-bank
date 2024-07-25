pub mod error;
mod repo;
mod value;

use crate::primitives::LoanTermsId;

pub use repo::*;
pub use value::*;

pub struct Terms {
    pub id: LoanTermsId,
    pub values: TermValues,
}

impl std::ops::Deref for Terms {
    type Target = TermValues;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
