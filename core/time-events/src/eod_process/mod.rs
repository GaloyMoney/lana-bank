mod entity;
pub mod error;
mod repo;

pub use entity::*;

use es_entity::clock::ClockHandle;
use obix::out::OutboxEventMarker;
use sqlx::PgPool;

use crate::{primitives::*, public::CoreEodEvent, publisher::EodPublisher};
use error::EodProcessError;
use repo::EodProcessRepo;

pub struct EodProcesses<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    repo: EodProcessRepo<E>,
}

impl<E> Clone for EodProcesses<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

impl<E> EodProcesses<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    pub fn new(pool: &PgPool, publisher: &EodPublisher<E>, clock: ClockHandle) -> Self {
        Self {
            repo: EodProcessRepo::new(pool, publisher, clock),
        }
    }

    pub async fn create_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        new_process: NewEodProcess,
    ) -> Result<EodProcess, EodProcessError> {
        let process = self.repo.create_in_op(op, new_process).await?;
        Ok(process)
    }

    pub async fn find_by_id(&self, id: EodProcessId) -> Result<EodProcess, EodProcessError> {
        Ok(self.repo.find_by_id(id).await?)
    }

    pub async fn find_by_id_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: EodProcessId,
    ) -> Result<EodProcess, EodProcessError> {
        Ok(self.repo.find_by_id_in_op(op, id).await?)
    }

    pub async fn update_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        process: &mut EodProcess,
    ) -> Result<(), EodProcessError> {
        self.repo.update_in_op(op, process).await?;
        Ok(())
    }

    pub async fn find_latest(&self) -> Result<Option<EodProcess>, EodProcessError> {
        let result = self
            .repo
            .list_by_date(
                es_entity::PaginatedQueryArgs {
                    first: 1,
                    after: None,
                },
                es_entity::ListDirection::Descending,
            )
            .await?;
        Ok(result.entities.into_iter().next())
    }
}
