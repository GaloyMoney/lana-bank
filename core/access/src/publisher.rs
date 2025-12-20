use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    CoreAccessEvent,
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
        entity: &User,
        new_events: es_entity::LastPersisted<'_, UserEvent>,
    ) -> Result<(), UserError> {
        use UserEvent::*;
        let events = new_events
            .map(|event| match &event.event {
                Initialized {
                    id, email, role_id, ..
                } => CoreAccessEvent::UserCreated {
                    id: *id,
                    email: email.clone(),
                    role_id: *role_id,
                },
                RoleUpdated { role_id, .. } => CoreAccessEvent::UserUpdatedRole {
                    id: entity.id,
                    role_id: *role_id,
                },
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }

    pub async fn publish_role(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Role,
        new_events: es_entity::LastPersisted<'_, RoleEvent>,
    ) -> Result<(), RoleError> {
        use RoleEvent::*;
        let events = new_events
            .map(|event| match &event.event {
                Initialized { id, name, .. } => CoreAccessEvent::RoleCreated {
                    id: *id,
                    name: name.clone(),
                },
                PermissionSetAdded {
                    permission_set_id, ..
                } => CoreAccessEvent::RoleGainedPermissionSet {
                    id: entity.id,
                    permission_set_id: *permission_set_id,
                },
                PermissionSetRemoved {
                    permission_set_id, ..
                } => CoreAccessEvent::RoleLostPermissionSet {
                    id: entity.id,
                    permission_set_id: *permission_set_id,
                },
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }
}
