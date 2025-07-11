use async_graphql::{Enum, SimpleObject};
use lana_app::report::{DagRunStatusResponse, ReportGenerateResponse};

use crate::primitives::Timestamp;

#[derive(SimpleObject)]
pub struct ReportGeneratePayload {
    pub run_id: String,
}

impl From<ReportGenerateResponse> for ReportGeneratePayload {
    fn from(response: ReportGenerateResponse) -> Self {
        Self {
            run_id: response.run_id,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RunType {
    Scheduled,
    ApiTriggered,
}

impl From<lana_app::report::RunType> for RunType {
    fn from(run_type: lana_app::report::RunType) -> Self {
        match run_type {
            lana_app::report::RunType::Scheduled => RunType::Scheduled,
            lana_app::report::RunType::ApiTriggered => RunType::ApiTriggered,
        }
    }
}

#[derive(SimpleObject)]
pub struct LastRun {
    pub run_type: RunType,
    pub run_started_at: Option<Timestamp>,
    pub status: String,
    pub logs: Option<String>,
}

impl From<lana_app::report::LastRun> for LastRun {
    fn from(last_run: lana_app::report::LastRun) -> Self {
        Self {
            run_type: last_run.run_type.into(),
            run_started_at: last_run.run_started_at.map(|dt| dt.into()),
            status: last_run.status,
            logs: last_run.logs,
        }
    }
}

#[derive(SimpleObject)]
pub struct ReportGenerationStatusPayload {
    pub running: bool,
    pub run_type: Option<RunType>,
    pub run_started_at: Option<Timestamp>,
    pub logs: Option<String>,
    pub last_run: Option<LastRun>,
    pub error: Option<String>,
}

impl From<DagRunStatusResponse> for ReportGenerationStatusPayload {
    fn from(response: DagRunStatusResponse) -> Self {
        Self {
            running: response.running,
            run_type: response.run_type.map(Into::into),
            run_started_at: response.run_started_at.map(|dt| dt.into()),
            logs: response.logs,
            last_run: response.last_run.map(Into::into),
            error: response.error,
        }
    }
}
