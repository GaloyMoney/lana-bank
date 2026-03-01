use async_graphql::*;

use admin_graphql_shared::primitives::*;

use super::Report;

pub use lana_app::report::{
    ReportRun as DomainReportRun, ReportRunState as DomainReportRunState,
    ReportRunType as DomainReportRunType, ReportRunsByCreatedAtCursor,
};

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
#[graphql(complex)]
pub struct ReportRun {
    id: ID,
    report_run_id: UUID,
    state: ReportRunState,
    run_type: ReportRunType,
    start_time: Option<Timestamp>,

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
            entity: Arc::new(report_run),
        }
    }
}

#[ComplexObject]
impl ReportRun {
    async fn reports(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Report>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let reports = app
            .reports()
            .list_reports_for_run(sub, self.entity.id)
            .await?;

        Ok(reports.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(SimpleObject)]
pub struct ReportRunCreatePayload {
    pub run_id: Option<String>,
}

#[derive(SimpleObject)]
pub struct ReportRunUpdatedPayload {
    pub report_run_id: UUID,
}
