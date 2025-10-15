#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod client;
pub mod config;
pub mod error;

pub use client::SmtpClient;
pub use config::SmtpConfig;
pub use error::SmtpError;
