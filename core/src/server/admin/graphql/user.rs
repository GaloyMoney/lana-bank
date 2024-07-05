use async_graphql::{types::connection::*, *};
use serde::{Deserialize, Serialize};

use crate::{
    primitives::{Satoshis, UserId},
    server::shared_graphql::{primitives::UUID, user::User},
};

#[derive(InputObject)]
pub struct UserPledgeCollateralInput {
    pub user_id: UUID,
    pub amount: Satoshis,
    pub reference: String,
}

#[derive(SimpleObject)]
pub struct UserPledgeCollateralPayload {
    pub user: User,
}

impl From<crate::user::User> for UserPledgeCollateralPayload {
    fn from(user: crate::user::User) -> Self {
        Self {
            user: User::from(user),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct UserByNameCursor {
    pub name: String,
    pub id: UserId,
}

impl CursorType for UserByNameCursor {
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

impl From<(UserId, &str)> for UserByNameCursor {
    fn from((id, name): (UserId, &str)) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

impl From<UserByNameCursor> for crate::user::UserByNameCursor {
    fn from(cursor: UserByNameCursor) -> Self {
        Self {
            id: cursor.id,
            name: cursor.name,
        }
    }
}

#[derive(InputObject)]
pub struct SumsubPermalinkCreateInput {
    pub user_id: String,
}
