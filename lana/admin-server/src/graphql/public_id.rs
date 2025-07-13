use async_graphql::Union;

use crate::graphql::{
    credit_facility::{CreditFacility, disbursal::CreditFacilityDisbursal},
    customer::Customer,
    deposit_account::DepositAccount,
};

#[derive(Union)]
pub enum PublicIdTarget {
    Customer(Customer),
    DepositAccount(DepositAccount),
    CreditFacility(CreditFacility),
    CreditFacilityDisbursal(CreditFacilityDisbursal),
}
