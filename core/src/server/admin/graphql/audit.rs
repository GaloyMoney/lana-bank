use async_graphql::{dataloader::DataLoader, ComplexObject, Context, SimpleObject, Union, ID};

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
    created_at: Timestamp,
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
            object: entry.object.as_ref().into(),
            action: entry.action.as_ref().into(),
            authorized: entry.authorized,
            created_at: entry.created_at.into(),
        }
    }
}

impl ToGlobalId for crate::primitives::AuditEntryId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("audit_entry:{}", self))
    }
}
