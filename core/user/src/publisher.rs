use outbox::{Outbox, OutboxEventMarker};

use crate::role::{error::RoleError, Role, RoleEvent};
use crate::CoreUserEvent;

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

    pub async fn publish_role(
        &self,
        _db: &mut es_entity::DbOp<'_>,
        _entity: &Role,
        _new_events: es_entity::LastPersisted<'_, RoleEvent>,
    ) -> Result<(), RoleError> {
        // use RoleEvent::*;
        // let events = new_events
        //     .filter_map(|event| match &event.event {
        //         Initialized { id, name } => Some(CoreUserEvent::RoleCreated {
        //             id: *id,
        //             name: name.clone(),
        //         }),
        //     })
        //     .collect::<Vec<_>>();

        // self.outbox.publish_all_persisted(db.tx(), events).await?;

        Ok(())
    }
}
