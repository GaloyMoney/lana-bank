use async_graphql::{
    ComplexObject, Context, ID, SimpleObject, Union, connection::CursorType, scalar,
};
use serde::{Deserialize, Serialize};

use admin_graphql_access::User;
use admin_graphql_shared::primitives::*;
use lana_app::primitives::Subject as DomainSubject;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditEntryId(audit::AuditEntryId);
scalar!(AuditEntryId);
impl From<audit::AuditEntryId> for AuditEntryId {
    fn from(value: audit::AuditEntryId) -> Self {
        Self(value)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditSubjectId(String);
scalar!(AuditSubjectId);
impl From<String> for AuditSubjectId {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<AuditSubjectId> for String {
    fn from(value: AuditSubjectId) -> Self {
        value.0
    }
}

#[derive(SimpleObject)]
pub struct System {
    actor: String,
}

impl System {
    pub fn from_actor(actor: &audit::SystemActor) -> Self {
        Self {
            actor: actor.to_string(),
        }
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
        let (app, _sub) = app_and_sub_from_ctx!(ctx);

        match &self.subject {
            DomainSubject::User(id) => {
                let mut users = app.access().users().find_all::<User>(&[*id]).await?;
                match users.remove(id) {
                    None => Err("User not found".into()),
                    Some(user) => Ok(AuditSubject::User(user)),
                }
            }
            DomainSubject::System(actor) => Ok(AuditSubject::System(System::from_actor(actor))),
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
