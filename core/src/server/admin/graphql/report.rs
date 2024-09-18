use async_graphql::*;

use crate::server::shared_graphql::primitives::UUID;

#[derive(SimpleObject)]
pub(super) struct ReportCreatePayload {
    report: Report,
}

#[derive(SimpleObject)]
pub(super) struct Report {
    report_id: UUID,
}

impl From<crate::report::Report> for ReportCreatePayload {
    fn from(report: crate::report::Report) -> Self {
        Self {
            report: Report::from(report),
        }
    }
}

impl From<crate::report::Report> for Report {
    fn from(report: crate::report::Report) -> Self {
        Self {
            report_id: UUID::from(report.id),
        }
    }
}
