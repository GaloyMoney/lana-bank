mod entity;
pub mod error;
mod repo;

#[cfg(feature = "json-schema")]
pub use entity::LiquidationProcessEvent;
pub(crate) use entity::*;
use error::LiquidationProcessError;
use outbox::OutboxEventMarker;
pub(crate) use repo::LiquidationProcessRepo;

use crate::{CoreCreditEvent, LiquidationProcessId};

pub struct Liquidations<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: LiquidationProcessRepo<E>,
}

impl<E> Liquidations<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &sqlx::PgPool, publisher: &crate::CreditFacilityPublisher<E>) -> Self {
        Self {
            repo: LiquidationProcessRepo::new(pool, publisher),
        }
    }

    pub async fn record_payment_from_liquidator(
        &self,
        liquidation_process_id: LiquidationProcessId,
    ) -> Result<(), LiquidationProcessError> {
        let mut liquidation = self.repo.find_by_id(liquidation_process_id).await?;

        // ???

        self.repo.update(&mut liquidation).await?;

        Ok(())
    }
}
