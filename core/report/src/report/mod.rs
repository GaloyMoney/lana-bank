mod entity;
pub mod error;
mod repo;

pub use entity::{NewReport, Report, ReportFile};

#[cfg(feature = "json-schema")]
pub use entity::ReportEvent;
pub use error::ReportError;
pub use repo::ReportRepo;

pub(crate) use repo::{ReportFindError, ReportQueryError};

pub use repo::report_cursor::*;
