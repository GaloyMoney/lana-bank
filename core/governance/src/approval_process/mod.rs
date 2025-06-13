mod entity;
pub mod error;
mod repo;

pub use entity::{ApprovalProcess, NewApprovalProcess};
#[cfg(feature = "json-schema")]
pub use entity::ApprovalProcessEvent;
pub use repo::approval_process_cursor;

pub(crate) use repo::ApprovalProcessRepo;
