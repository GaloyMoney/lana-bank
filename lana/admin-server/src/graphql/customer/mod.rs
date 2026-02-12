use async_graphql::*;

use crate::primitives::*;
use lana_app::public_id::PublicId;

use super::{
    credit_facility::*, deposit_account::*, document::CustomerDocument, primitives::SortDirection,
};

pub use lana_app::customer::{
    Activity, Customer as DomainCustomer, CustomerType, CustomersCursor,
    CustomersFilters as DomainCustomersFilters, CustomersSortBy as DomainCustomersSortBy, KycLevel,
    KycVerification, Sort,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Customer {
    id: ID,
    customer_id: UUID,
    kyc_verification: KycVerification,
    activity: Activity,
    level: KycLevel,
    created_at: Timestamp,
    customer_type: CustomerType,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainCustomer>,
}

impl From<DomainCustomer> for Customer {
    fn from(customer: DomainCustomer) -> Self {
        Customer {
            id: customer.id.to_global_id(),
            customer_id: UUID::from(customer.id),
            kyc_verification: customer.kyc_verification,
            activity: customer.activity,
            level: customer.level,
            created_at: customer.created_at().into(),
            customer_type: customer.customer_type,
            entity: Arc::new(customer),
        }
    }
}

#[ComplexObject]
impl Customer {
    async fn email(&self) -> &str {
        &self.entity.email
    }

    async fn telegram_handle(&self) -> &str {
        &self.entity.telegram_handle
    }

    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn applicant_id(&self) -> Option<&str> {
        self.entity.applicant_id.as_deref()
    }

    async fn deposit_account(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<DepositAccount>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        Ok(app
            .deposits()
            .list_accounts_by_created_at_for_account_holder(
                sub,
                self.customer_id,
                Default::default(),
                ListDirection::Descending,
            )
            .await?
            .entities
            .into_iter()
            .map(DepositAccount::from)
            .next())
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacility>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let credit_facilities: Vec<CreditFacility> = app
            .credit()
            .facilities()
            .list(
                sub,
                Default::default(),
                DomainCreditFacilitiesFilters {
                    customer_id: Some(self.entity.id),
                    ..Default::default()
                },
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

    async fn pending_credit_facilities(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<crate::graphql::credit_facility::PendingCreditFacility>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let proposals = app
            .credit()
            .pending_credit_facilities()
            .list_for_customer_by_created_at(sub, self.entity.id)
            .await?
            .into_iter()
            .map(crate::graphql::credit_facility::PendingCreditFacility::from)
            .collect();

        Ok(proposals)
    }

    async fn credit_facility_proposals(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<crate::graphql::credit_facility::CreditFacilityProposal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let proposals = app
            .credit()
            .proposals()
            .list_for_customer_by_created_at(sub, self.entity.id)
            .await?
            .into_iter()
            .map(crate::graphql::credit_facility::CreditFacilityProposal::from)
            .collect();

        Ok(proposals)
    }

    async fn documents(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<CustomerDocument>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let documents = app
            .customers()
            .list_documents_for_customer_id(sub, self.entity.id)
            .await?;
        Ok(documents.into_iter().map(CustomerDocument::from).collect())
    }

    async fn user_can_create_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().subject_can_create(sub, false).await.is_ok())
    }
}

#[derive(InputObject)]
pub struct CustomerCreateInput {
    pub email: String,
    pub telegram_handle: String,
    pub customer_type: CustomerType,
}
crate::mutation_payload! { CustomerCreatePayload, customer: Customer }

#[derive(InputObject)]
pub struct CustomerTelegramHandleUpdateInput {
    pub customer_id: UUID,
    pub telegram_handle: String,
}
crate::mutation_payload! { CustomerTelegramHandleUpdatePayload, customer: Customer }

#[derive(InputObject)]
pub struct CustomerEmailUpdateInput {
    pub customer_id: UUID,
    pub email: String,
}
crate::mutation_payload! { CustomerEmailUpdatePayload, customer: Customer }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustomersSortBy {
    CreatedAt,
    #[default]
    Email,
    TelegramHandle,
}

impl From<CustomersSortBy> for DomainCustomersSortBy {
    fn from(by: CustomersSortBy) -> Self {
        match by {
            CustomersSortBy::CreatedAt => DomainCustomersSortBy::CreatedAt,
            CustomersSortBy::Email => DomainCustomersSortBy::Email,
            CustomersSortBy::TelegramHandle => DomainCustomersSortBy::TelegramHandle,
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

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CustomerKycUpdatedPayload {
    pub kyc_verification: KycVerification,
    #[graphql(skip)]
    pub customer_id: CustomerId,
}

#[ComplexObject]
impl CustomerKycUpdatedPayload {
    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<super::loader::LanaDataLoader>();
        let customer = loader
            .load_one(self.customer_id)
            .await?
            .expect("Customer not found");
        Ok(customer)
    }
}
