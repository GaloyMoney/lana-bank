use async_graphql::*;
use std::sync::Arc;

use lana_app::customer::{
    Customer as DomainCustomer, CustomerStatus, CustomerType, KycLevel, PersonalInfo,
};

use crate::primitives::*;

use super::{credit_facility::*, deposit_account::*};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerError {
    #[error("CustomerError - DepositAccountNotFound")]
    DepositAccountNotFound,
}

/// The authenticated customer record.
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
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let party = app
            .customers()
            .find_party_by_customer_id_without_audit(self.entity.id)
            .await?;
        Ok(party.email)
    }

    /// Telegram handle associated with the customer.
    async fn telegram_handle(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let party = app
            .customers()
            .find_party_by_customer_id_without_audit(self.entity.id)
            .await?;
        Ok(party.telegram_handle)
    }

    /// Operational customer type for this customer.
    async fn customer_type(&self, ctx: &Context<'_>) -> async_graphql::Result<CustomerType> {
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let party = app
            .customers()
            .find_party_by_customer_id_without_audit(self.entity.id)
            .await?;
        Ok(party.customer_type)
    }

    /// Personal information collected for this customer, if available.
    async fn personal_info(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<PersonalInfo>> {
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let party = app
            .customers()
            .find_party_by_customer_id_without_audit(self.entity.id)
            .await?;
        Ok(party.personal_info)
    }

    /// The most recently created deposit account available to this customer.
    async fn deposit_account(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccount> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        Ok(app
            .deposits()
            .for_subject(sub)?
            .list_accounts_by_created_at(Default::default(), ListDirection::Descending)
            .await?
            .entities
            .into_iter()
            .map(DepositAccount::from)
            .next()
            .ok_or(CustomerError::DepositAccountNotFound)?)
    }

    /// Credit facilities visible to this customer, newest first.
    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacility>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        Ok(app
            .credit()
            .for_subject(sub)?
            .list_credit_facilities_by_created_at(Default::default(), ListDirection::Descending)
            .await?
            .entities
            .into_iter()
            .map(CreditFacility::from)
            .collect())
    }
}
