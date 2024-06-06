#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod bank;
pub mod cli;
pub mod entity;
pub mod fixed_term_loan;
pub mod job;
pub mod ledger;
pub mod primitives;
pub mod server;
pub mod user;
pub mod withdraw;
