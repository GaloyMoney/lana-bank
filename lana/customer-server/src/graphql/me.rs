use async_graphql::*;

use lana_app::customer::Customer as DomainCustomer;

use super::customer::*;

#[derive(SimpleObject)]
#[graphql(name = "Me")]
pub(super) struct MeCustomer {
    customer: Customer,
}

impl From<DomainCustomer> for MeCustomer {
    fn from(entity: DomainCustomer) -> Self {
        Self {
            customer: Customer::from(entity),
        }
    }
}
