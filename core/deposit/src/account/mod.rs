mod entity;
pub mod error;
mod repo;

pub use entity::DepositAccount;
#[cfg(feature = "json-schema")]
pub use entity::DepositAccountEvent;
pub(crate) use entity::*;
pub(crate) use repo::*;
