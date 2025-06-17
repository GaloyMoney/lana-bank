use sqlx::PgPool;

use es_entity::*;
use outbox::OutboxEventMarker;

use crate::{event::CoreDocumentStorageEvent, primitives::*, publisher::*};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Document",
    err = "DocumentStorageError",
    columns(owner_id(ty = "Option<DocumentOwnerId>", list_for, update(persist = false))),
    tbl_prefix = "core",
    post_persist_hook = "publish"
)]
pub struct DocumentRepo<E>
where
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    pool: PgPool,
    publisher: DocumentStoragePublisher<E>,
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
    pub(super) fn new(pool: &PgPool, publisher: &DocumentStoragePublisher<E>) -> Self {
        Self {
            pool: pool.clone(),
            publisher: publisher.clone(),
        }
    }

    async fn publish(
        &self,
        db: &mut es_entity::DbOp<'_>,
        entity: &Document,
        new_events: es_entity::LastPersisted<'_, DocumentEvent>,
    ) -> Result<(), DocumentStorageError> {
        self.publisher.publish(db, entity, new_events).await
    }
}
