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
                } => Some(CoreAccessEvent::UserCreated {
                    entity: PublicUser {
                        id: *id,
                        email: email.clone(),
                        role_id: *role_id,
                    },
                }),
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
                Initialized { id, name, .. } => Some(CoreAccessEvent::RoleCreated {
                    entity: PublicRole {
                        id: *id,
                        name: name.clone(),
                    },
                }),
                PermissionSetAdded { .. } => None,
                PermissionSetRemoved { .. } => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }
}
