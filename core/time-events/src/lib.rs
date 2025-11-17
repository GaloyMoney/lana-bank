#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod broadcaster;
mod config;
pub mod error;
mod event;
mod primitives;

use tracing::instrument;

use outbox::{Outbox, OutboxEventMarker};

pub use broadcaster::DailyClosingBroadcaster;
pub use config::TimeEventsConfig;
pub use error::TimeEventsError;
pub use event::TimeEvent;
pub use primitives::*;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::event::TimeEvent;
}

pub struct TimeEvents<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    _outbox: Outbox<E>,
}

impl<E> Clone for TimeEvents<E>
where
    E: OutboxEventMarker<TimeEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
        }
    }
}

impl<E> TimeEvents<E>
where
    E: OutboxEventMarker<TimeEvent> + Send + Sync + 'static,
{
    #[instrument(name = "time_events.init", skip(outbox))]
    pub fn init(outbox: &Outbox<E>, config: TimeEventsConfig) -> Self {
        let broadcaster = DailyClosingBroadcaster::new(outbox, config);

        tokio::spawn(async move {
            broadcaster.run().await;
        });

        Self {
            _outbox: outbox.clone(),
        }
    }
}
