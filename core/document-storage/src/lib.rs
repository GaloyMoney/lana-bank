#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod event;
mod primitives;
mod publisher;
mod repo;


use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

pub use entity::Document;
use error::*;
pub use event::*;
pub use primitives::*;
pub use repo::{DocumentRepo};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DocumentEvent;
}

use publisher::*;

pub struct DocumentStorage<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    authz: Perms,
    outbox: Outbox<E>,
    repo: DocumentRepo<E>,
}

impl<Perms, E> Clone for DocumentStorage<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Perms, E> DocumentStorage<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<DocumentStorageObject>,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, outbox: &Outbox<E>) -> Self {
        let publisher = DocumentStoragePublisher::new(outbox);
        let repo = DocumentRepo::new(pool, &publisher);
        Self {
            repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
        }
    }

    pub async fn upload(&self) -> Result<(), DocumentStorageError> {
        todo!()
    }
}