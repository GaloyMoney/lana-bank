use serde::{Deserialize, Serialize};

use super::{Loan, LoanId};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoanCursor {
    pub id: LoanId,
}

impl From<&Loan> for LoanCursor {
    fn from(values: &Loan) -> Self {
        Self { id: values.id }
    }
}
