#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod bfx_client;
pub mod error;
pub mod event;
pub mod job;
mod primitives;

use core_money::UsdCents;
use outbox::Outbox;
use outbox::OutboxEventMarker;

use crate::job::{UpdateBtcPriceInit, UpdateBtcPriceJobConfig};
use ::job::Jobs;
use error::PriceError;
pub use event::PriceUpdated;
use futures::StreamExt;
pub use primitives::*;

#[derive(Clone)]
pub struct Price<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    outbox: Outbox<E>,
}

impl<E> Price<E>
where
    E: OutboxEventMarker<PriceUpdated>,
{
    pub async fn init(jobs: &Jobs, outbox: Outbox<E>) -> Result<Self, PriceError> {
        jobs.add_initializer_and_spawn_unique(
            UpdateBtcPriceInit::new(&outbox),
            UpdateBtcPriceJobConfig::new(),
        )
        .await?;

        Ok(Self { outbox })
    }
}
