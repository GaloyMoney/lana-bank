use async_graphql::{types::connection::*, *};
use serde::{Deserialize, Serialize};

use crate::{
    primitives::CustomerId,
    server::shared_graphql::{customer::Customer, primitives::UUID},
};

#[derive(InputObject)]
pub struct CustomerCreateInput {
    pub email: String,
    pub telegram_id: String,
}

#[derive(InputObject)]

pub struct CustomerUpdateInput {
    pub customer_id: UUID,
    pub telegram_id: String,
}

#[derive(SimpleObject)]
pub struct CustomerUpdatePayload {
    pub customer: Customer,
}

impl From<crate::customer::Customer> for CustomerUpdatePayload {
    fn from(customer: crate::customer::Customer) -> Self {
        Self {
            customer: Customer::from(customer),
        }
    }
}

#[derive(SimpleObject)]
pub struct CustomerCreatePayload {
    pub customer: Customer,
}

impl From<crate::customer::Customer> for CustomerCreatePayload {
    fn from(customer: crate::customer::Customer) -> Self {
        Self {
            customer: Customer::from(customer),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct CustomerByEmailCursor {
    pub email: String,
    pub id: CustomerId,
}

impl CursorType for CustomerByEmailCursor {
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

impl From<(CustomerId, &str)> for CustomerByEmailCursor {
    fn from((id, email): (CustomerId, &str)) -> Self {
        Self {
            id,
            email: email.to_string(),
        }
    }
}

impl From<CustomerByEmailCursor> for crate::customer::CustomerByEmailCursor {
    fn from(cursor: CustomerByEmailCursor) -> Self {
        Self {
            id: cursor.id,
            email: cursor.email,
        }
    }
}

#[derive(InputObject)]
pub struct SumsubPermalinkCreateInput {
    pub customer_id: UUID,
}
