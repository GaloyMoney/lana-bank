use obix::out::{Outbox, OutboxEventMarker};

use crate::{
    CoreAccessEvent, PublicAgent, PublicRole, PublicUser,
    agent::{Agent, AgentEvent},
    role::{Role, RoleEvent},
    user::{User, UserEvent},
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

    pub async fn publish_user_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &User,
        new_events: es_entity::LastPersisted<'_, UserEvent>,
    ) -> Result<(), sqlx::Error> {
        let events = new_events
            .filter_map(|event| match &event.event {
                UserEvent::Initialized { .. } => Some(CoreAccessEvent::UserCreated {
                    entity: PublicUser::from(entity),
                }),
                UserEvent::RoleUpdated { .. } => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }

    pub async fn publish_role_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Role,
        new_events: es_entity::LastPersisted<'_, RoleEvent>,
    ) -> Result<(), sqlx::Error> {
        let events = new_events
            .filter_map(|event| match &event.event {
                RoleEvent::Initialized { .. } => Some(CoreAccessEvent::RoleCreated {
                    entity: PublicRole::from(entity),
                }),
                RoleEvent::PermissionSetAdded { .. } => None,
                RoleEvent::PermissionSetRemoved { .. } => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }
}

pub struct AgentPublisher<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
}

impl<E> Clone for AgentPublisher<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            outbox: self.outbox.clone(),
        }
    }
}

impl<E> AgentPublisher<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(outbox: &Outbox<E>) -> Self {
        Self {
            outbox: outbox.clone(),
        }
    }

    pub async fn publish_agent_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Agent,
        new_events: es_entity::LastPersisted<'_, AgentEvent>,
    ) -> Result<(), sqlx::Error> {
        let events = new_events
            .filter_map(|event| match &event.event {
                AgentEvent::Initialized { .. } => Some(CoreAccessEvent::AgentCreated {
                    entity: PublicAgent::from(entity),
                }),
                AgentEvent::Deactivated => None,
                AgentEvent::Reactivated => None,
            })
            .collect::<Vec<_>>();

        self.outbox.publish_all_persisted(op, events).await?;

        Ok(())
    }
}
