#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod broadcaster;
mod config;
pub mod error;
mod event;
pub mod job;
mod primitives;
mod time;

pub use broadcaster::DailyClosingBroadcaster;
pub use config::TimeEventsConfig;
pub use error::TimeEventsError;
pub use event::TimeEvent;
pub use job::{DailyClosingBroadcasterInit, DailyClosingBroadcasterJobConfig};
pub use primitives::*;

#[derive(Debug, Clone, Copy)]
pub struct TimeEvents;

impl TimeEvents {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TimeEvents {
    fn default() -> Self {
        Self::new()
    }
}
