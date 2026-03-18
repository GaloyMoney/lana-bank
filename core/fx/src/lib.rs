#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub(crate) mod chart_of_accounts_integration;
pub mod error;
pub(crate) mod ledger;
mod primitives;

pub use primitives::*;
