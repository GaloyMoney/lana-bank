mod entity;
pub mod error;
mod repo;

pub use entity::PaymentAllocation;

#[cfg(feature = "json-schema")]
pub use entity::PaymentAllocationEvent;
pub(super) use entity::*;
pub(crate) use repo::PaymentAllocationRepo;
