pub mod error;

use crate::{ledger::*, primitives::UsdCents};

use error::OwnersEquityError;

#[derive(Clone)]
pub struct OwnersEquity {
    ledger: Ledger,
}

impl OwnersEquity {
    pub fn new(ledger: &Ledger) -> Self {
        Self {
            ledger: ledger.clone(),
        }
    }

    pub async fn add_equity(
        &self,
        amount: UsdCents,
        reference: String,
    ) -> Result<(), OwnersEquityError> {
        Ok(self.ledger.add_equity(amount, reference).await?)
    }
}
