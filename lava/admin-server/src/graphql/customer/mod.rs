mod balance;

use async_graphql::{connection::*, *};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

use super::{credit_facility::*, deposit::*, document::Document, withdrawal::Withdrawal};

pub use lava_app::{
    app::LavaApp,
    customer::{
        Customer as DomainCustomer, CustomerByCreatedAtCursor, CustomerByEmailCursor,
        CustomerByTelegramIdCursor,
    },
};

pub use balance::*;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Customer {
    id: ID,
    customer_id: UUID,
    status: AccountStatus,
    level: KycLevel,

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

    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<CustomerBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app
            .ledger()
            .get_customer_balance(self.entity.account_ids)
            .await?;
        Ok(CustomerBalance::from(balance))
    }

    async fn deposits(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Deposit>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let deposits = app
            .deposits()
            .list_for_customer(sub, self.entity.id)
            .await?
            .into_iter()
            .map(Deposit::from)
            .collect();
        Ok(deposits)
    }

    async fn withdrawals(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Withdrawal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let withdraws = app
            .withdrawals()
            .list_for_customer(sub, self.entity.id)
            .await?
            .into_iter()
            .map(Withdrawal::from)
            .collect();
        Ok(withdraws)
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacility>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let credit_facilities: Vec<CreditFacility> = app
            .credit_facilities()
            .list_for_customer(sub, self.entity.id)
            .await?
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

    async fn subject_can_record_deposit(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.deposits().subject_can_record(sub, false).await.is_ok())
    }

    async fn subject_can_initiate_withdrawal(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .withdrawals()
            .subject_can_initiate(sub, false)
            .await
            .is_ok())
    }
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

#[derive(Serialize, Deserialize)]
pub enum CustomerCursor {
    ByEmail(CustomerByEmailCursor),
    ByCreatedAt(CustomerByCreatedAtCursor),
    ByTelegramId(CustomerByTelegramIdCursor),
}

impl CursorType for CustomerCursor {
    type Error = String;
    fn encode_cursor(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let json = serde_json::to_string(&self).expect("could not serialize token");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }
    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

impl TryFrom<CustomerCursor> for CustomerByEmailCursor {
    type Error = String;
    fn try_from(cursor: CustomerCursor) -> Result<Self, Self::Error> {
        match cursor {
            CustomerCursor::ByEmail(cursor) => Ok(cursor),
            _ => Err("Invalid combo cursor variant".to_string()),
        }
    }
}

impl TryFrom<CustomerCursor> for CustomerByCreatedAtCursor {
    type Error = String;
    fn try_from(cursor: CustomerCursor) -> Result<Self, Self::Error> {
        match cursor {
            CustomerCursor::ByCreatedAt(cursor) => Ok(cursor),
            _ => Err("Invalid combo cursor variant".to_string()),
        }
    }
}

impl TryFrom<CustomerCursor> for CustomerByTelegramIdCursor {
    type Error = String;
    fn try_from(cursor: CustomerCursor) -> Result<Self, Self::Error> {
        match cursor {
            CustomerCursor::ByTelegramId(cursor) => Ok(cursor),
            _ => Err("Invalid combo cursor variant".to_string()),
        }
    }
}

impl From<CustomerByEmailCursor> for CustomerCursor {
    fn from(cursor: CustomerByEmailCursor) -> Self {
        CustomerCursor::ByEmail(cursor)
    }
}

impl From<CustomerByCreatedAtCursor> for CustomerCursor {
    fn from(cursor: CustomerByCreatedAtCursor) -> Self {
        CustomerCursor::ByCreatedAt(cursor)
    }
}

impl From<CustomerByTelegramIdCursor> for CustomerCursor {
    fn from(cursor: CustomerByTelegramIdCursor) -> Self {
        CustomerCursor::ByTelegramId(cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_customer_cursor() {
        let cursor = CustomerCursor::ByEmail(CustomerByEmailCursor {
            id: CustomerId::new(),
            email: "user@example.com".to_string(),
        });

        let encoded = cursor.encode_cursor();
        let decoded = CustomerCursor::decode_cursor(&encoded).expect("Failed to decode cursor");

        match (cursor, decoded) {
            (CustomerCursor::ByEmail(original_cursor), CustomerCursor::ByEmail(decoded_cursor)) => {
                assert_eq!(original_cursor.email, decoded_cursor.email,);
            }
            _ => panic!("Decoded cursor is not of type ByEmail"),
        }
    }
}
