use async_graphql::{connection::CursorType, ComplexObject, Context, SimpleObject, Union, ID};
use serde::{Deserialize, Serialize};

use crate::primitives::*;
use lana_app::primitives::Subject as DomainSubject;

use super::{loader::*, user::User};

#[derive(SimpleObject)]
pub struct System {
    name: &'static str,
}

#[derive(Union)]
enum AuditSubject {
    User(User),
    System(System),
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct AuditEntry {
    id: ID,
    object: String,
    action: String,
    authorized: bool,
    recorded_at: Timestamp,

    #[graphql(skip)]
    subject: DomainSubject,
}

#[ComplexObject]
impl AuditEntry {
    async fn subject(&self, ctx: &Context<'_>) -> async_graphql::Result<AuditSubject> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();

        match self.subject {
            DomainSubject::User(id) => {
                let user = loader.load_one(id).await?;
                match user {
                    None => Err("User not found".into()),
                    Some(user) => Ok(AuditSubject::User(user)),
                }
            }
            DomainSubject::System => {
                let system = System { name: "lana" };
                Ok(AuditSubject::System(system))
            }
            DomainSubject::Customer(_) => {
                panic!("Whoops - have we gone live yet?");
            }
        }
    }
}

impl From<lana_app::audit::AuditEntry> for AuditEntry {
    fn from(entry: lana_app::audit::AuditEntry) -> Self {
        Self {
            id: entry.id.to_global_id(),
            subject: entry.subject,
            object: entry.object.to_string(),
            action: entry.action.to_string(),
            authorized: entry.authorized,
            recorded_at: entry.recorded_at.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuditCursor {
    id: audit::AuditEntryId,
}

impl From<&lana_app::audit::AuditEntry> for AuditCursor {
    fn from(entry: &lana_app::audit::AuditEntry) -> Self {
        Self { id: entry.id }
    }
}
impl From<AuditCursor> for lana_app::audit::AuditCursor {
    fn from(cursor: AuditCursor) -> Self {
        Self { id: cursor.id }
    }
}

impl CursorType for AuditCursor {
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
