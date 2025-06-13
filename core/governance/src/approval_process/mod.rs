mod entity;
pub mod error;
mod repo;

#[cfg(feature = "json-schema")]
pub use entity::ApprovalProcessEvent;
pub use entity::{ApprovalProcess, NewApprovalProcess};
pub use repo::approval_process_cursor;

pub(crate) use repo::ApprovalProcessRepo;
