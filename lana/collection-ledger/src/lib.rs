#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod collection_ledger;
mod templates;

pub use collection_ledger::CollectionLedger;
