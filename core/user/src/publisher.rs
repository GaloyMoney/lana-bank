use outbox::{Outbox, OutboxEventMarker};

use crate::{
    role::{error::RoleError, Role, RoleEvent},
    user::{error::UserError, User, UserEvent},
    CoreUserEvent,
};

pub struct UserPublisher<E>
where
    E: OutboxEventMarker<CoreUserEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for UserPublisher<E>
where
    E: OutboxEventMarker<CoreUserEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> UserPublisher<E>
where
    E: OutboxEventMarker<CoreUserEvent>,
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
                Initialized { id, email, .. } => Some(CoreUserEvent::UserCreated {
                    id: *id,
                    email: email.clone(),
                }),
                RoleAssigned { role, .. } => Some(CoreUserEvent::UserGrantedRole {
                    id: entity.id,
                    role: role.clone(),
                }),
                RoleRevoked { role, .. } => Some(CoreUserEvent::UserRevokedRole {
                    id: entity.id,
                    role: role.clone(),
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
            .filter_map(|event| match &event.event {
                Initialized { id, name } => Some(CoreUserEvent::RoleCreated {
                    id: *id,
                    name: name.clone(),
                }),
                PermissionAdded { object, action } => Some(CoreUserEvent::RoleGainedPermission {
                    id: entity.id,
                    object: object.clone(),
                    action: action.clone(),
                }),
                PermissionRemoved { object, action } => Some(CoreUserEvent::RoleLostPermission {
                    id: entity.id,
                    object: object.clone(),
                    action: action.clone(),
                }),
                GainedInheritanceFrom { .. } => None,
                LostInheritanceFrom { .. } => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(db.tx(), events).await?;

        Ok(())
    }
}
