use async_graphql::Union;

use crate::graphql::{
    credit_facility::{CreditFacility, disbursal::CreditFacilityDisbursal},
    customer::Customer,
    deposit::Deposit,
    deposit_account::DepositAccount,
    prospect::Prospect,
    withdrawal::Withdrawal,
};

#[derive(Union)]
pub enum PublicIdTarget {
    Customer(Customer),
    Prospect(Prospect),
    DepositAccount(DepositAccount),
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    CreditFacility(CreditFacility),
    CreditFacilityDisbursal(CreditFacilityDisbursal),
}
