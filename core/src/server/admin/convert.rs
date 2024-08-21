use async_graphql::connection::CursorType;

use crate::audit::AuditCursor;

impl CursorType for AuditCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        self.to_string()
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let cursor = s.parse::<Self>().map_err(|e| e.to_string())?;
        Ok(cursor)
    }
}
