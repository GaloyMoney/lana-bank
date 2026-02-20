pub use admin_graphql_deposit::DepositBase;

pub use super::deposit_account::DepositAccount;

// Type alias - DepositBase already has #[graphql(name = "Deposit")]
pub type Deposit = DepositBase;
