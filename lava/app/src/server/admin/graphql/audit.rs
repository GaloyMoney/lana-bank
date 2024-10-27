use async_graphql::{
    connection::CursorType, dataloader::DataLoader, ComplexObject, Context, SimpleObject, Union, ID,
};
use serde::{Deserialize, Serialize};

use crate::{
    primitives::Subject as DomainSubject,
    server::shared_graphql::{convert::*, customer::Customer, primitives::Timestamp},
};

use super::{loader::LavaDataLoader, user::User};

#[derive(SimpleObject)]
pub struct System {
    name: String,
}

#[derive(Union)]
enum Subject {
    User(User),
    Customer(Customer),
    System(System),
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct AuditEntry {
    id: ID,
    #[graphql(skip)]
    subject: DomainSubject,
    object: String,
    action: String,
    authorized: bool,
    recorded_at: Timestamp,
}

#[ComplexObject]
impl AuditEntry {
    async fn subject(&self, ctx: &Context<'_>) -> async_graphql::Result<Subject> {
        let loader = ctx.data_unchecked::<DataLoader<LavaDataLoader>>();

        match self.subject {
            DomainSubject::User(id) => {
                let user = loader.load_one(id).await?;
                match user {
                    None => Err("User not found".into()),
                    Some(user) => Ok(Subject::User(user)),
                }
            }
            DomainSubject::Customer(id) => {
                let customer = loader.load_one(id).await?;
                match customer {
                    None => Err("Customer not found".into()),
                    Some(customer) => Ok(Subject::Customer(customer)),
                }
            }
            DomainSubject::System(node) => {
                let system = System {
                    // FIXME: this is the ID, also return name of the node
                    name: node.to_string(),
                };
                Ok(Subject::System(system))
            }
        }
    }
}

impl From<crate::audit::AuditEntry> for AuditEntry {
    fn from(entry: crate::audit::AuditEntry) -> Self {
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

impl ToGlobalId for crate::audit::AuditEntryId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("audit_entry:{}", self))
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuditCursor {
    id: crate::audit::AuditEntryId,
}

impl From<&crate::audit::AuditEntry> for AuditCursor {
    fn from(entry: &crate::audit::AuditEntry) -> Self {
        Self { id: entry.id }
    }
}
impl From<AuditCursor> for crate::audit::AuditCursor {
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
