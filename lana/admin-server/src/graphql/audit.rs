use async_graphql::{ComplexObject, Context, ID, SimpleObject, Union, connection::CursorType};
use serde::{Deserialize, Serialize};

use crate::primitives::*;
use lana_app::primitives::Subject as DomainSubject;

use super::{access::User, loader::*};

#[derive(SimpleObject)]
pub struct System {
    name: String,
}

impl System {
    pub const SYSTEM_NAME: &'static str = "lana";

    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn system() -> Self {
        Self::new(Self::SYSTEM_NAME)
    }
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
    audit_entry_id: AuditEntryId,
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
            DomainSubject::System => Ok(AuditSubject::System(System::system())),
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
            audit_entry_id: entry.id.into(),
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
        use base64::{Engine as _, engine::general_purpose};
        let json = serde_json::to_string(&self).expect("could not serialize token");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}
