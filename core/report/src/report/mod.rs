mod entity;
pub mod error;
mod repo;

pub use entity::{NewReport, Report, ReportFile};

#[cfg(feature = "json-schema")]
pub use entity::ReportEvent;
#[cfg(not(feature = "json-schema"))]
pub(crate) use entity::ReportEvent;
pub use error::ReportError;
pub(super) use repo::ReportRepo;

pub use repo::report_cursor::*;
