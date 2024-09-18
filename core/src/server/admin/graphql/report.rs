use async_graphql::*;

use crate::{primitives::ReportProgress, server::shared_graphql::primitives::UUID};

#[derive(SimpleObject)]
pub(super) struct ReportCreatePayload {
    report: Report,
}

#[derive(SimpleObject)]
pub(super) struct Report {
    report_id: UUID,
    progress: ReportProgress,
}

pub(super) struct ReportDownloadLink {
    report_id: UUID,
    report_name: String,
    url: String,
}

pub(super) struct ReportDownloadLinksGeneratePayload {
    links: Vec<ReportDownloaLink>,
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
            progress: report.progress(),
        }
    }
}
