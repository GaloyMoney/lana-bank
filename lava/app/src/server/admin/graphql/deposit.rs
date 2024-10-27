use async_graphql::{types::connection::*, *};

use crate::{
    primitives::UsdCents,
    server::shared_graphql::{deposit::Deposit, primitives::*},
};

#[derive(InputObject)]
pub struct DepositRecordInput {
    pub customer_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}

#[derive(SimpleObject)]
pub struct DepositRecordPayload {
    pub deposit: Deposit,
}

impl From<crate::deposit::Deposit> for DepositRecordPayload {
    fn from(deposit: crate::deposit::Deposit) -> Self {
        Self {
            deposit: Deposit::from(deposit),
        }
    }
}

pub use crate::deposit::DepositByCreatedAtCursor;
impl CursorType for DepositByCreatedAtCursor {
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
