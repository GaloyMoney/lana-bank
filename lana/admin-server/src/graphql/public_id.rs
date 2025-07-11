use async_graphql::Union;

use crate::graphql::{customer::Customer, deposit_account::DepositAccount};

#[derive(Union)]
pub enum PublicIdTarget {
    Customer(Customer),
    DepositAccount(DepositAccount),
}
