use async_graphql::*;

use core_customer::Customer as DomainCustomer;

use super::customer::*;

#[derive(SimpleObject)]
#[graphql(name = "Me")]
pub struct MeCustomer {
    customer: Customer,
}

impl From<DomainCustomer> for MeCustomer {
    fn from(entity: DomainCustomer) -> Self {
        Self {
            customer: Customer::from(entity),
        }
    }
}
