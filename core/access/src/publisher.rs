use audit::AuditInfo;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    CoreAccessEvent, PublicRole, PublicUser,
    role::{Role, RoleEvent, error::RoleError},
    user::{User, UserEvent, error::UserError},
};

pub struct UserPublisher<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for UserPublisher<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> UserPublisher<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_user(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        _entity: &User,
        new_events: es_entity::LastPersisted<'_, UserEvent>,
    ) -> Result<(), UserError> {
        use UserEvent::*;
        let events = new_events
            .filter_map(|event| match &event.event {
                Initialized {
                    id, email, role_id, ..
                } => {
                    let created_by = extract_created_by(&event.context);
                    Some(CoreAccessEvent::UserCreated {
                        entity: PublicUser {
                            id: *id,
                            email: email.clone(),
                            role_id: *role_id,
                            created_at: event.recorded_at,
                            created_by,
                        },
                    })
                }
                RoleUpdated { .. } => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }

    pub async fn publish_role(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        _entity: &Role,
        new_events: es_entity::LastPersisted<'_, RoleEvent>,
    ) -> Result<(), RoleError> {
        use RoleEvent::*;
        let events = new_events
            .filter_map(|event| match &event.event {
                Initialized { id, name, .. } => {
                    let created_by = extract_created_by(&event.context);
                    Some(CoreAccessEvent::RoleCreated {
                        entity: PublicRole {
                            id: *id,
                            name: name.clone(),
                            created_at: event.recorded_at,
                            created_by,
                        },
                    })
                }
                PermissionSetAdded { .. } => None,
                PermissionSetRemoved { .. } => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }
}

fn extract_created_by(context: &Option<es_entity::ContextData>) -> String {
    context
        .as_ref()
        .and_then(|ctx| ctx.lookup::<AuditInfo>("audit_info").ok().flatten())
        .map(|info| info.sub)
        .unwrap_or_else(|| "unknown".to_string())
}
