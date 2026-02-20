pub use admin_graphql_deposit::DepositAccountBase;

// Type alias - DepositAccountBase already has #[graphql(name = "DepositAccount")]
pub type DepositAccount = DepositAccountBase;
