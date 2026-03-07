use serde::{Deserialize, Serialize};

use crate::primitives::UserId;

use super::entity::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCursor {
    pub id: UserId,
}

impl From<&User> for UserCursor {
    fn from(user: &User) -> Self {
        Self { id: user.id }
    }
}

#[cfg(feature = "graphql")]
impl es_entity::graphql::async_graphql::connection::CursorType for UserCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{Engine as _, engine::general_purpose};
        let json = serde_json::to_string(&self).expect("could not serialize cursor");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e: base64::DecodeError| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e: serde_json::Error| e.to_string())
    }
}

#[cfg(all(test, feature = "graphql"))]
mod tests {
    use super::*;
    use es_entity::graphql::async_graphql::connection::CursorType;

    #[test]
    fn test_user_cursor_encoding_decoding() {
        let id = UserId::new();
        let cursor = UserCursor { id };

        let encoded = cursor.encode_cursor();
        let decoded = UserCursor::decode_cursor(&encoded).unwrap();

        assert_eq!(cursor.id, decoded.id);
    }
}
