use async_graphql::*;

use crate::primitives::*;
use lana_app::public_id::PublicId;

use super::{
    credit_facility::*, deposit_account::*, document::CustomerDocument, loader::LanaDataLoader,
    primitives::SortDirection,
};

pub use lana_app::customer::{
    Customer as DomainCustomer, CustomerConversion, CustomerStatus, CustomerType, CustomersCursor,
    CustomersFilters as DomainCustomersFilters, CustomersSortBy as DomainCustomersSortBy, KycLevel,
    PersonalInfo, Sort,
};

/// Describes how a prospect became a customer.
#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionReason {
    /// Converted automatically after Sumsub approved the applicant.
    SumsubApproved,
    /// Converted manually by an admin action.
    ManuallyConverted,
}

impl From<&CustomerConversion> for ConversionReason {
    fn from(conversion: &CustomerConversion) -> Self {
        match conversion {
            CustomerConversion::SumsubApproved { .. } => ConversionReason::SumsubApproved,
            CustomerConversion::ManuallyConverted => ConversionReason::ManuallyConverted,
        }
    }
}

/// A customer record exposed in the admin API.
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Customer {
    /// Relay global identifier for this customer.
    id: ID,
    /// Internal UUID for this customer.
    customer_id: UUID,
    /// Current lifecycle status of the customer.
    status: CustomerStatus,
    /// Current KYC level for the customer.
    level: KycLevel,
    /// When the customer record was created.
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
    /// Email address associated with the customer.
    async fn email(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.email.clone())
    }

    /// Telegram handle associated with the customer.
    async fn telegram_handle(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.telegram_handle.clone())
    }

    /// Public identifier assigned to the customer.
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    /// Sumsub applicant identifier, if the customer has entered KYC.
    async fn applicant_id(&self) -> Option<&str> {
        self.entity.applicant_id()
    }

    /// How this customer was converted from a prospect.
    async fn conversion_reason(&self) -> ConversionReason {
        ConversionReason::from(&self.entity.conversion)
    }

    /// Operational customer type for this customer.
    async fn customer_type(&self, ctx: &Context<'_>) -> async_graphql::Result<CustomerType> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.customer_type)
    }

    /// Personal information collected for this customer, if available.
    async fn personal_info(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<PersonalInfo>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.personal_info.clone())
    }

    /// The most recently created deposit account for this customer, if any.
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

    /// Credit facilities for this customer, newest first.
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

    /// Pending credit facilities for this customer, newest first.
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

    /// Credit facility proposals for this customer, newest first.
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

    /// Documents attached to this customer.
    async fn documents(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<CustomerDocument>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let documents = app
            .customers()
            .list_documents_for_customer_id(sub, self.entity.id)
            .await?;
        Ok(documents.into_iter().map(CustomerDocument::from).collect())
    }

    /// Whether the current admin can create a new credit facility for this customer.
    async fn user_can_create_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.credit().subject_can_create(sub, false).await.is_ok())
    }
}

/// Input for updating a customer's Telegram handle.
#[derive(InputObject)]
pub struct CustomerTelegramHandleUpdateInput {
    /// Internal UUID of the customer to update.
    pub customer_id: UUID,
    /// New Telegram handle to store for the customer.
    pub telegram_handle: String,
}
crate::mutation_payload! { CustomerTelegramHandleUpdatePayload, customer: Customer }

/// Input for updating a customer's email address.
#[derive(InputObject)]
pub struct CustomerEmailUpdateInput {
    /// Internal UUID of the customer to update.
    pub customer_id: UUID,
    /// New email address to store for the customer.
    pub email: String,
}
crate::mutation_payload! { CustomerEmailUpdatePayload, customer: Customer }

/// Input for freezing a customer.
#[derive(InputObject)]
pub struct CustomerFreezeInput {
    /// Internal UUID of the customer to freeze.
    pub customer_id: UUID,
}
crate::mutation_payload! { CustomerFreezePayload, customer: Customer }

/// Input for unfreezing a customer.
#[derive(InputObject)]
pub struct CustomerUnfreezeInput {
    /// Internal UUID of the customer to unfreeze.
    pub customer_id: UUID,
}
crate::mutation_payload! { CustomerUnfreezePayload, customer: Customer }

/// Input for closing a customer.
#[derive(InputObject)]
pub struct CustomerCloseInput {
    /// Internal UUID of the customer to close.
    pub customer_id: UUID,
}
crate::mutation_payload! { CustomerClosePayload, customer: Customer }

/// Fields available when sorting customer lists.
#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CustomersSortBy {
    /// Sort by when the customer was created.
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

/// Sort options for customer lists.
#[derive(InputObject, Default, Clone, Copy)]
pub struct CustomersSort {
    /// Field to sort customers by.
    #[graphql(default)]
    pub by: CustomersSortBy,
    /// Direction to apply to the selected sort field.
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

/// Filters that can be applied when listing customers.
#[derive(InputObject)]
pub struct CustomersFilter {
    /// Limit results to a specific customer type.
    pub customer_type: Option<CustomerType>,
    /// Limit results to a specific customer status.
    pub status: Option<CustomerStatus>,
}
