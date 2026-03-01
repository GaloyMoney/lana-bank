use async_graphql::*;

use admin_graphql_shared::primitives::*;

pub use lana_app::report::{Report as DomainReport, ReportFile as DomainReportFile};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Report {
    id: ID,
    report_id: UUID,
    external_id: String,
    name: String,
    norm: String,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainReport>,
}

impl From<lana_app::report::Report> for Report {
    fn from(report: lana_app::report::Report) -> Self {
        Report {
            id: report.id.to_global_id(),
            created_at: report.created_at().into(),
            report_id: UUID::from(report.id),
            external_id: report.external_id.clone(),
            name: report.name.clone(),
            norm: report.norm.clone(),
            entity: Arc::new(report),
        }
    }
}

#[derive(SimpleObject)]
pub struct ReportFile {
    extension: String,
}

impl From<DomainReportFile> for ReportFile {
    fn from(file: DomainReportFile) -> Self {
        ReportFile {
            extension: file.extension,
        }
    }
}

#[ComplexObject]
impl Report {
    async fn run_id(&self) -> UUID {
        UUID::from(self.entity.run_id)
    }

    async fn files(&self) -> Vec<ReportFile> {
        self.entity
            .files
            .iter()
            .map(|f| ReportFile::from(f.clone()))
            .collect()
    }

    async fn report_run(&self, ctx: &Context<'_>) -> async_graphql::Result<super::ReportRun> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let report_run = app
            .reports()
            .find_report_run_by_id(sub, self.entity.run_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("report run not found"))?;
        Ok(report_run.into())
    }
}

#[derive(SimpleObject)]
pub struct ReportFileGenerateDownloadLinkPayload {
    pub url: String,
}

#[derive(InputObject)]
pub struct ReportFileGenerateDownloadLinkInput {
    pub report_id: UUID,
    pub extension: String,
}
