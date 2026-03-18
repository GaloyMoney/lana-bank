use async_graphql::*;

use crate::primitives::*;

use super::{super::primitives::SortDirection, report::Report};

pub use lana_app::report::{
    ReportRun as DomainReportRun, ReportRunState as DomainReportRunState,
    ReportRunType as DomainReportRunType, ReportRunsByCreatedAtCursor,
    RequestedReport as DomainRequestedReport,
};

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportRunsSortBy {
    #[default]
    StartTime,
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct ReportRunsSort {
    #[graphql(default)]
    pub by: ReportRunsSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReportRunState {
    Queued,
    Running,
    Success,
    Failed,
}

impl From<DomainReportRunState> for ReportRunState {
    fn from(state: DomainReportRunState) -> Self {
        match state {
            DomainReportRunState::Queued => ReportRunState::Queued,
            DomainReportRunState::Running => ReportRunState::Running,
            DomainReportRunState::Success => ReportRunState::Success,
            DomainReportRunState::Failed => ReportRunState::Failed,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReportRunType {
    Scheduled,
    Manual,
}

impl From<DomainReportRunType> for ReportRunType {
    fn from(run_type: DomainReportRunType) -> Self {
        match run_type {
            DomainReportRunType::Scheduled => ReportRunType::Scheduled,
            DomainReportRunType::Manual => ReportRunType::Manual,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct RequestedReport {
    report_definition_id: String,
    norm: String,
    name: String,
}

impl From<DomainRequestedReport> for RequestedReport {
    fn from(requested_report: DomainRequestedReport) -> Self {
        Self {
            report_definition_id: requested_report.report_definition_id.to_string(),
            norm: requested_report.norm,
            name: requested_report.name,
        }
    }
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct ReportRun {
    id: ID,
    report_run_id: UUID,
    state: ReportRunState,
    run_type: ReportRunType,
    start_time: Option<Timestamp>,
    requested_report: Option<RequestedReport>,
    requested_as_of_date: Option<Date>,

    #[graphql(skip)]
    pub entity: Arc<DomainReportRun>,
}

impl From<lana_app::report::ReportRun> for ReportRun {
    fn from(report_run: lana_app::report::ReportRun) -> Self {
        ReportRun {
            id: report_run.id.to_global_id(),
            report_run_id: UUID::from(report_run.id),
            state: ReportRunState::from(report_run.state),
            run_type: ReportRunType::from(report_run.run_type),
            start_time: report_run.start_time.map(|dt| dt.into()),
            requested_report: report_run
                .requested_report
                .clone()
                .map(RequestedReport::from),
            requested_as_of_date: report_run.requested_as_of_date.map(Into::into),
            entity: Arc::new(report_run),
        }
    }
}

#[ComplexObject]
impl ReportRun {
    async fn reports(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Report>> {
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let sub = &ctx
            .data_unchecked::<crate::primitives::AdminAuthContext>()
            .sub;

        let reports = app
            .reports()
            .list_reports_for_run(sub, self.entity.id)
            .await?;

        Ok(reports.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(SimpleObject)]
pub struct ReportRunCreatePayload {
    pub run_id: String,
}

#[derive(InputObject)]
pub struct TriggerReportRunInput {
    pub report_definition_id: String,
    pub as_of_date: Option<Date>,
}

#[derive(SimpleObject)]
pub struct ReportRunUpdatedPayload {
    pub report_run_id: UUID,
}
