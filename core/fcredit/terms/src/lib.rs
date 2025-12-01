#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod balance_summary;
pub mod collateralization;
mod cvl;
mod effective_date;
mod error;
mod value;

pub use value::{InterestPeriod, TermValues};
