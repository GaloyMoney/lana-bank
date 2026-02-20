pub use admin_graphql_deposit::WithdrawalBase;

// Type alias - WithdrawalBase already has #[graphql(name = "Withdrawal")]
pub type Withdrawal = WithdrawalBase;
