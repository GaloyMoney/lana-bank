use async_graphql::Union;

use crate::graphql::customer::Customer;

#[derive(Union)]
pub enum PublicIdTarget {
    Customer(Customer),
}
