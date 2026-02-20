pub use admin_graphql_customer::CustomerBase;

// Type alias - CustomerBase already has #[graphql(name = "Customer")]
pub type Customer = CustomerBase;
