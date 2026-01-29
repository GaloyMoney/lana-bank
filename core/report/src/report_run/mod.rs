mod entity;
pub mod error;
mod repo;

pub use entity::{NewReportRun, ReportRun, ReportRunState, ReportRunType};

#[cfg(feature = "json-schema")]
pub use entity::ReportRunEvent;
#[cfg(not(feature = "json-schema"))]
pub(crate) use entity::ReportRunEvent;
pub use error::ReportRunError;
pub use repo::ReportRunRepo;

pub use repo::report_run_cursor::*;
