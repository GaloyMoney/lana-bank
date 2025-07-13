use async_graphql::Union;

use crate::graphql::{customer::Customer, deposit_account::DepositAccount, credit_facility::CreditFacility};

#[derive(Union)]
pub enum PublicIdTarget {
    Customer(Customer),
    DepositAccount(DepositAccount),
    CreditFacility(CreditFacility),
}
