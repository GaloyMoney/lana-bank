pub mod entity;
pub mod error;
mod repo;

use es_entity::clock::ClockHandle;

use crate::primitives::FxPositionId;

pub use entity::*;
use error::FxPositionError;
use repo::FxPositionRepo;

#[derive(Clone)]
pub struct FxPositions {
    repo: FxPositionRepo,
}

impl FxPositions {
    pub fn new(pool: &sqlx::PgPool, clock: ClockHandle) -> Self {
        Self {
            repo: FxPositionRepo::new(pool, clock),
        }
    }

    /// Find or create a position for the given currency.
    pub async fn find_or_create_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        currency: &str,
    ) -> Result<FxPosition, FxPositionError> {
        match self
            .repo
            .maybe_find_by_currency_in_op(&mut *op, currency.to_string())
            .await?
        {
            Some(position) => Ok(position),
            None => {
                let new = NewFxPosition::builder()
                    .id(FxPositionId::new())
                    .currency(currency.to_string())
                    .build()
                    .expect("Could not build NewFxPosition");
                let position = self.repo.create_in_op(op, new).await?;
                Ok(position)
            }
        }
    }

    pub async fn update_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        position: &mut FxPosition,
    ) -> Result<(), FxPositionError> {
        self.repo.update_in_op(op, position).await?;
        Ok(())
    }
}
