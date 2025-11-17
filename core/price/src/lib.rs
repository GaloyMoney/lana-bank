#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod bfx_client;
mod cache;
pub mod error;
mod event;
pub mod jobs;
mod primitives;

use error::PriceError;
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};

pub use cache::PriceCache;
pub use event::*;
pub use primitives::*;

#[derive(Clone)]
pub struct Price {
    cache: PriceCache,
}

impl Price {
    pub fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceError> {
        Ok(self.cache.get_price())
    }

    pub async fn init<E>(jobs: &Jobs, outbox: &Outbox<E>) -> Result<Self, PriceError>
    where
        E: OutboxEventMarker<CorePriceEvent>,
    {
        // this has to change , either give no price available error or source it from outbox on startup
        let cache = PriceCache::new(PriceOfOneBTC::ZERO);

        jobs.add_initializer_and_spawn_unique(
            jobs::get_price_from_bfx::GetPriceFromClientJobInit::<E>::new(outbox),
            jobs::get_price_from_bfx::GetPriceFromClientJobConfig::<E> {
                _phantom: std::marker::PhantomData,
            },
        )
        .await
        .map_err(PriceError::JobError)?;

        jobs.add_initializer_and_spawn_unique(
            jobs::update_price_in_cache::UpdatePriceInCacheInit::<E>::new(outbox, &cache),
            jobs::update_price_in_cache::UpdatePriceInCacheJobConfig::<E> {
                _phantom: std::marker::PhantomData,
            },
        )
        .await
        .map_err(PriceError::JobError)?;

        Ok(Self { cache })
    }
}
