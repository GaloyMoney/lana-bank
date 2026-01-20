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
        let events = new_events
            .filter_map(|event| PublicUser::try_from(event).ok())
            .map(|entity| CoreAccessEvent::UserCreated { entity })
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
        let events = new_events
            .filter_map(|event| PublicRole::try_from(event).ok())
            .map(|entity| CoreAccessEvent::RoleCreated { entity })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }
}
