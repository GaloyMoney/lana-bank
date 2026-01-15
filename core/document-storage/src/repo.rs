use sqlx::PgPool;

use es_entity::*;
use obix::out::OutboxEventMarker;

use crate::event::CoreDocumentStorageEvent;
use crate::primitives::*;
use crate::publisher::DocumentPublisher;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Document",
    err = "DocumentStorageError",
    columns(reference_id(ty = "ReferenceId", list_for, update(persist = false))),
    tbl_prefix = "core",
    delete = "soft",
    post_persist_hook = "publish"
)]
pub struct DocumentRepo<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    pool: PgPool,
    publisher: DocumentPublisher<E>,
}

impl<E> Clone for DocumentRepo<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            publisher: self.publisher.clone(),
        }
    }
}

impl<E> DocumentRepo<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    pub(super) fn new(pool: &PgPool, publisher: &DocumentPublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entity: &Document,
        new_events: es_entity::LastPersisted<'_, DocumentEvent>,
    ) -> Result<(), DocumentStorageError> {
        self.publisher.publish(op, entity, new_events).await
    }
}
