pub mod error;

use crate::{ledger::*, primitives::UsdCents};

use error::OwnersEquityError;
use serde_json::Value as JsonValue;

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
        let metadata = JsonValue::from(r#"{"Hello": "World"}"#);
        Ok(self.ledger.add_equity(amount, reference, metadata).await?)
    }
}
