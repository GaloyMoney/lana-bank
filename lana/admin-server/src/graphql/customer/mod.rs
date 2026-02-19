use async_graphql::*;

use crate::primitives::*;

use super::{
    credit_facility::*, deposit_account::*, document::CustomerDocument, loader::LanaDataLoader,
};

pub use admin_graphql_customer::{
    CustomerBase, CustomerEmailUpdateInput, CustomerTelegramHandleUpdateInput, CustomersCursor,
    CustomersFilter, CustomersSort, DomainCustomer, DomainCustomersFilters, DomainCustomersSortBy,
    ListDirection,
};

// ===== Customer =====

#[derive(Clone)]
pub(super) struct CustomerCrossDomain {
    entity: Arc<DomainCustomer>,
}

#[Object]
impl CustomerCrossDomain {
    async fn email(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.email.clone())
    }

    async fn telegram_handle(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.telegram_handle.clone())
    }

    async fn customer_type(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<lana_app::customer::CustomerType> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.customer_type)
    }

    async fn personal_info(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<lana_app::customer::PersonalInfo>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.personal_info.clone())
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
                self.entity.id,
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
                admin_graphql_credit::Sort {
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
    ) -> async_graphql::Result<Vec<PendingCreditFacility>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let proposals = app
            .credit()
            .pending_credit_facilities()
            .list_for_customer_by_created_at(sub, self.entity.id)
            .await?
            .into_iter()
            .map(PendingCreditFacility::from)
            .collect();

        Ok(proposals)
    }

    async fn credit_facility_proposals(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityProposal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let proposals = app
            .credit()
            .proposals()
            .list_for_customer_by_created_at(sub, self.entity.id)
            .await?
            .into_iter()
            .map(CreditFacilityProposal::from)
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

#[derive(MergedObject, Clone)]
#[graphql(name = "Customer")]
pub struct Customer(pub CustomerBase, CustomerCrossDomain);

impl From<DomainCustomer> for Customer {
    fn from(customer: DomainCustomer) -> Self {
        let base = CustomerBase::from(customer);
        let cross = CustomerCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for Customer {
    type Target = CustomerBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { CustomerTelegramHandleUpdatePayload, customer: Customer }
crate::mutation_payload! { CustomerEmailUpdatePayload, customer: Customer }
