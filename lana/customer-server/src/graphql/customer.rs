use async_graphql::*;
use std::sync::Arc;

use lana_app::customer::{Customer as DomainCustomer, CustomerType, KycLevel, KycVerification};

use crate::primitives::*;

use super::{credit_facility::*, deposit_account::*};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomerError {
    #[error("CustomerError - DepositAccountNotFound")]
    DepositAccountNotFound,
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Customer {
    id: ID,
    customer_id: UUID,
    customer_type: CustomerType,
    kyc_verification: KycVerification,
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
            customer_type: customer.customer_type,
            kyc_verification: customer.kyc_verification,
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

    async fn telegram_handle(&self) -> &str {
        &self.entity.telegram_handle
    }

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
