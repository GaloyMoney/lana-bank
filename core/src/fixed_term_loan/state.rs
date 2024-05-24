use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, async_graphql::Enum)]
pub enum FixedTermLoanState {
    Initializing,
    PendingCollateralization,
    Collateralized,
}
