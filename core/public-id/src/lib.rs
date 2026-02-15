#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use std::collections::HashMap;
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use entity::{NewPublicIdEntity, PublicIdEntity};
pub use error::*;
pub use primitives::*;
pub use repo::PublicIdRepo;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::PublicIdEntityEvent;
}

pub struct PublicIds {
    repo: PublicIdRepo,
}

impl Clone for PublicIds {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

impl PublicIds {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = PublicIdRepo::new(pool);
        Self { repo }
    }

    #[record_error_severity]
    #[instrument(name = "public_id_service.create_in_op", skip(self, op))]
    pub async fn create_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        target_type: impl Into<PublicIdTargetType> + std::fmt::Debug,
        target_id: impl Into<PublicIdTargetId> + std::fmt::Debug,
    ) -> Result<PublicIdEntity, PublicIdError> {
        let target_id = target_id.into();
        let reference = self.repo.next_counter().await?;

        let new_public_id = NewPublicIdEntity::builder()
            .id(reference)
            .target_id(target_id)
            .target_type(target_type)
            .build()
            .expect("Could not build public id");

        let public_id = self.repo.create_in_op(op, new_public_id).await?;
        Ok(public_id)
    }

    #[record_error_severity]
    #[instrument(name = "public_id_service.update_target_in_op", skip(self, op))]
    pub async fn update_target_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        id: &PublicId,
        target_type: impl Into<PublicIdTargetType> + std::fmt::Debug,
        target_id: impl Into<PublicIdTargetId> + std::fmt::Debug,
    ) -> Result<(), PublicIdError> {
        let mut public_id_entity = self.repo.find_by_id(id.clone()).await?;
        if public_id_entity
            .update_target(target_id, target_type)
            .did_execute()
        {
            self.repo.update_in_op(op, &mut public_id_entity).await?;
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "public_id_service.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        id: impl Into<PublicId> + std::fmt::Debug,
    ) -> Result<Option<PublicIdEntity>, PublicIdError> {
        self.repo.maybe_find_by_id(id.into()).await
    }

    #[record_error_severity]
    #[instrument(name = "public_id_service.find_all", skip(self))]
    pub async fn find_all<T: From<PublicIdEntity>>(
        &self,
        ids: &[PublicId],
    ) -> Result<HashMap<PublicId, T>, PublicIdError> {
        self.repo.find_all(ids).await
    }
}
