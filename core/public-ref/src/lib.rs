#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use tracing::instrument;

pub use entity::{NewPublicRef, PublicRef};
use error::*;
pub use primitives::*;
pub use repo::{public_ref_cursor::PublicRefsByCreatedAtCursor, PublicRefRepo};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::PublicRefEvent;
}

pub struct PublicRefService {
    repo: PublicRefRepo,
}

impl Clone for PublicRefService {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

impl PublicRefService {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = PublicRefRepo::new(pool);
        Self { repo }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, sqlx::Error> {
        self.repo.begin_op().await
    }

    #[instrument(name = "public_ref_service.create_in_op", skip(self, db), err)]
    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
    ) -> Result<PublicRef, PublicRefError> {
        let public_ref_id = PublicRefId::new();

        let new_public_ref = NewPublicRef::builder()
            .id(public_ref_id)
            .build()
            .expect("Could not build public ref");

        let public_ref = self.repo.create_in_op(db, new_public_ref).await?;
        Ok(public_ref)
    }
}