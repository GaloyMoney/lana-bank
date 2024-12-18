mod error;

use async_graphql::*;

use crate::primitives::*;

use super::{
    credit_facility::*, deposit_account::*, document::Document, primitives::SortDirection,
};

pub use lana_app::{
    app::LanaApp,
    customer::{
        Customer as DomainCustomer, CustomersCursor, CustomersSortBy as DomainCustomersSortBy,
        FindManyCustomers, Sort,
    },
};

pub use error::*;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Customer {
    id: ID,
    customer_id: UUID,
    status: AccountStatus,
    level: KycLevel,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainCustomer>,
}

impl From<DomainCustomer> for Customer {
    fn from(customer: DomainCustomer) -> Self {
        Customer {
            id: customer.id.to_global_id(),
            customer_id: UUID::from(customer.id),
            status: customer.status,
            level: customer.level,
            created_at: customer.created_at().into(),
            entity: Arc::new(customer),
        }
    }
}

#[ComplexObject]
impl Customer {
    async fn email(&self) -> &str {
        &self.entity.email
    }

    async fn telegram_id(&self) -> &str {
        &self.entity.telegram_id
    }

    async fn applicant_id(&self) -> Option<&str> {
        self.entity.applicant_id.as_deref()
    }

    async fn deposit_account(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccount> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let account = app
            .deposits()
            .find_account_for_account_holder(sub, self.customer_id)
            .await?
            .expect("deposit account should exist for a customer");
        Ok(DepositAccount::from(account))
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacility>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let credit_facilities: Vec<CreditFacility> = app
            .credit_facilities()
            .list(
                sub,
                Default::default(),
                FindManyCreditFacilities::WithCustomerId(self.entity.id),
                Sort {
                    by: DomainCreditFacilitiesSortBy::CreatedAt,
                    direction: ListDirection::Descending,
                },
            )
            .await?
            .entities
            .into_iter()
            .map(CreditFacility::from)
            .collect();

        Ok(credit_facilities)
    }

    async fn documents(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Document>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let documents = app
            .documents()
            .list_for_customer_id(sub, self.entity.id)
            .await?;
        Ok(documents.into_iter().map(Document::from).collect())
    }

    async fn subject_can_create_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit_facilities()
            .subject_can_create(sub, false)
            .await
            .is_ok())
    }

    // async fn subject_can_record_deposit(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
    //     let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
    //     Ok(app.deposits().subject_can_record(sub, false).await.is_ok())
    // }

    // async fn subject_can_initiate_withdrawal(
    //     &self,
    //     ctx: &Context<'_>,
    // ) -> async_graphql::Result<bool> {
    //     let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
    //     Ok(app
    //         .withdrawals()
    //         .subject_can_initiate(sub, false)
    //         .await
    //         .is_ok())
    // }
}

#[derive(InputObject)]
pub struct CustomerCreateInput {
    pub email: String,
    pub telegram_id: String,
}
crate::mutation_payload! { CustomerCreatePayload, customer: Customer }

#[derive(InputObject)]
pub struct CustomerUpdateInput {
    pub customer_id: UUID,
    pub telegram_id: String,
}
crate::mutation_payload! { CustomerUpdatePayload, customer: Customer }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustomersSortBy {
    CreatedAt,
    #[default]
    Email,
    TelegramId,
}

impl From<CustomersSortBy> for DomainCustomersSortBy {
    fn from(by: CustomersSortBy) -> Self {
        match by {
            CustomersSortBy::CreatedAt => DomainCustomersSortBy::CreatedAt,
            CustomersSortBy::Email => DomainCustomersSortBy::Email,
            CustomersSortBy::TelegramId => DomainCustomersSortBy::TelegramId,
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

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomersFilterBy {
    AccountStatus,
}

#[derive(InputObject)]
pub struct CustomersFilter {
    pub field: CustomersFilterBy,
    pub status: Option<AccountStatus>,
}
