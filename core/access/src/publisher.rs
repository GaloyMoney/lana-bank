use outbox::{Outbox, OutboxEventMarker};

use crate::{
    role::{error::RoleError, Role, RoleEvent},
    user::{error::UserError, User, UserEvent},
    CoreAccessEvent,
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
        db: &mut es_entity::DbOp<'_>,
        entity: &User,
        new_events: es_entity::LastPersisted<'_, UserEvent>,
    ) -> Result<(), UserError> {
        use UserEvent::*;
        let events = new_events
            .filter_map(|event| match &event.event {
                Initialized { id, email, .. } => Some(CoreAccessEvent::UserCreated {
                    id: *id,
                    email: email.clone(),
                }),
                RoleGranted { role_id: id, .. } => Some(CoreAccessEvent::UserGrantedRole {
                    id: entity.id,
                    role_id: *id,
                }),
                RoleRevoked { role_id: id, .. } => Some(CoreAccessEvent::UserRevokedRole {
                    id: entity.id,
                    role_id: *id,
                }),
                AuthenticationIdUpdated { .. } => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(db.tx(), events).await?;

        Ok(())
    }

    pub async fn publish_role(
        &self,
        db: &mut es_entity::DbOp<'_>,
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

        self.outbox.publish_all_persisted(db.tx(), events).await?;

        Ok(())
    }
}
