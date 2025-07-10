#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use std::collections::HashMap;
use tracing::instrument;

pub use entity::{NewPublicRef, PublicRef};
use error::*;
pub use primitives::*;
pub use repo::{public_ref_cursor::PublicRefsByCreatedAtCursor, PublicRefRepo};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::PublicRefEvent;
}

pub struct PublicRefs {
    repo: PublicRefRepo,
}

impl Clone for PublicRefs {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

impl PublicRefs {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = PublicRefRepo::new(pool);
        Self { repo }
    }

    #[instrument(name = "public_ref_service.create_in_op", skip(self, db), err)]
    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        target_type: impl Into<RefTargetType> + std::fmt::Debug,
        target_id: impl Into<RefTargetId> + std::fmt::Debug,
    ) -> Result<PublicRef, PublicRefError> {
        let target_id = target_id.into();
        let counter = self.repo.next_counter().await?;
        let reference = Ref::from_counter(counter);

        let new_public_ref = NewPublicRef::builder()
            .id(reference)
            .target_id(target_id)
            .target_type(target_type)
            .build()
            .expect("Could not build public ref");

        let public_ref = self.repo.create_in_op(db, new_public_ref).await?;
        Ok(public_ref)
    }

    #[instrument(name = "public_ref_service.find_by_reference", skip(self), err)]
    pub async fn find_by_reference(
        &self,
        reference: impl Into<Ref> + std::fmt::Debug,
    ) -> Result<PublicRef, PublicRefError> {
        self.repo.find_by_id(reference.into()).await
    }

    #[instrument(name = "public_ref_service.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        reference: impl Into<Ref> + std::fmt::Debug,
    ) -> Result<PublicRef, PublicRefError> {
        self.repo.find_by_id(reference.into()).await
    }

    #[instrument(name = "public_ref_service.find_all", skip(self), err)]
    pub async fn find_all<T: From<PublicRef>>(
        &self,
        references: &[Ref],
    ) -> Result<HashMap<Ref, T>, PublicRefError> {
        self.repo.find_all(references).await
    }
}
