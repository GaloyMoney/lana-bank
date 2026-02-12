use async_graphql::Union;

use crate::graphql::{
    credit_facility::{CreditFacility, disbursal::CreditFacilityDisbursal},
    customer::Customer,
    deposit::Deposit,
    deposit_account::DepositAccount,
    withdrawal::Withdrawal,
};

#[derive(Union)]
pub(super) enum PublicIdTarget {
    Customer(Customer),
    DepositAccount(DepositAccount),
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    CreditFacility(CreditFacility),
    CreditFacilityDisbursal(CreditFacilityDisbursal),
}
