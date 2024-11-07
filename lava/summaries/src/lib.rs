#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod customer;

use lava_events::LavaEvent;

pub type Outbox = outbox::Outbox<LavaEvent>;
