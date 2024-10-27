use async_graphql::{types::connection::*, *};
use serde::{Deserialize, Serialize};

use crate::{
    primitives::{UsdCents, WithdrawId},
    server::shared_graphql::{primitives::UUID, withdraw::Withdrawal},
};

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub customer_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}

#[derive(SimpleObject)]
pub struct WithdrawalInitiatePayload {
    pub withdrawal: Withdrawal,
}

impl From<crate::withdraw::Withdraw> for WithdrawalInitiatePayload {
    fn from(withdrawal: crate::withdraw::Withdraw) -> Self {
        Self {
            withdrawal: Withdrawal::from(withdrawal),
        }
    }
}

#[derive(InputObject)]
pub struct WithdrawalConfirmInput {
    pub withdrawal_id: UUID,
}

#[derive(SimpleObject)]
pub struct WithdrawalConfirmPayload {
    pub withdrawal: Withdrawal,
}

impl From<crate::withdraw::Withdraw> for WithdrawalConfirmPayload {
    fn from(withdrawal: crate::withdraw::Withdraw) -> Self {
        Self {
            withdrawal: Withdrawal::from(withdrawal),
        }
    }
}

#[derive(InputObject)]
pub struct WithdrawalCancelInput {
    pub withdrawal_id: UUID,
}

#[derive(SimpleObject)]
pub struct WithdrawalCancelPayload {
    pub withdrawal: Withdrawal,
}

impl From<crate::withdraw::Withdraw> for WithdrawalCancelPayload {
    fn from(withdrawal: crate::withdraw::Withdraw) -> Self {
        Self {
            withdrawal: Withdrawal::from(withdrawal),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct WithdrawByIdCursor {
    pub id: WithdrawId,
}

impl CursorType for WithdrawByIdCursor {
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

impl From<WithdrawId> for WithdrawByIdCursor {
    fn from(id: WithdrawId) -> Self {
        Self { id }
    }
}

impl From<WithdrawByIdCursor> for crate::withdraw::WithdrawByIdCursor {
    fn from(cursor: WithdrawByIdCursor) -> Self {
        Self { id: cursor.id }
    }
}
