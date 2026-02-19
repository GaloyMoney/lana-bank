use async_graphql::*;

use crate::primitives::*;

pub use lana_app::{
    credit::ListDirection,
    customer::{
        Activity, Customer as DomainCustomer, CustomersCursor,
        CustomersFilters as DomainCustomersFilters, CustomersSortBy as DomainCustomersSortBy,
        KycLevel, KycVerification, Sort,
    },
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CustomerBase {
    id: ID,
    customer_id: UUID,
    kyc_verification: KycVerification,
    activity: Activity,
    level: KycLevel,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainCustomer>,
}

impl From<DomainCustomer> for CustomerBase {
    fn from(customer: DomainCustomer) -> Self {
        CustomerBase {
            id: customer.id.to_global_id(),
            customer_id: UUID::from(customer.id),
            kyc_verification: customer.kyc_verification,
            activity: customer.activity,
            level: customer.level,
            created_at: customer.created_at().into(),
            entity: Arc::new(customer),
        }
    }
}

#[ComplexObject]
impl CustomerBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn applicant_id(&self) -> &str {
        &self.entity.applicant_id
    }
}

#[derive(InputObject)]
pub struct CustomerTelegramHandleUpdateInput {
    pub customer_id: UUID,
    pub telegram_handle: String,
}

#[derive(InputObject)]
pub struct CustomerEmailUpdateInput {
    pub customer_id: UUID,
    pub email: String,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustomersSortBy {
    #[default]
    CreatedAt,
}

impl From<CustomersSortBy> for DomainCustomersSortBy {
    fn from(by: CustomersSortBy) -> Self {
        match by {
            CustomersSortBy::CreatedAt => DomainCustomersSortBy::CreatedAt,
        }
    }
}

#[derive(InputObject, Default, Clone, Copy)]
pub struct CustomersSort {
    #[graphql(default)]
    pub by: CustomersSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<CustomersSort> for DomainCustomersSortBy {
    fn from(sort: CustomersSort) -> Self {
        sort.by.into()
    }
}

impl From<CustomersSort> for Sort<DomainCustomersSortBy> {
    fn from(sort: CustomersSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

#[derive(InputObject)]
pub struct CustomersFilter {
    pub kyc_verification: Option<KycVerification>,
}
