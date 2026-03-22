use async_graphql::*;

use crate::primitives::*;

use super::super::loader::LanaDataLoader;

pub use lana_app::report::{Report as DomainReport, ReportFile as DomainReportFile};

#[derive(SimpleObject, Clone)]
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("reportId".to_string())
)]
pub struct Report {
    report_id: ReportId,
    external_id: String,
    name: String,
    norm: String,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainReport>,
}

impl From<lana_app::report::Report> for Report {
    fn from(report: lana_app::report::Report) -> Self {
        Report {
            created_at: report.created_at().into(),
            report_id: report.id,
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
    async fn run_id(&self) -> ReportRunId {
        self.entity.run_id
    }

    async fn files(&self) -> Vec<ReportFile> {
        self.entity
            .files
            .iter()
            .map(|f| ReportFile::from(f.clone()))
            .collect()
    }

    async fn report_run(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<super::report_run::ReportRun> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let report_run = loader
            .load_one(self.entity.run_id)
            .await?
            .ok_or_else(|| Error::new("Report run not found"))?;
        Ok(report_run)
    }
}

#[derive(SimpleObject)]
pub struct ReportFileDownloadLinkGeneratePayload {
    pub url: String,
}

#[derive(InputObject)]
pub struct ReportFileDownloadLinkGenerateInput {
    pub report_id: ReportId,
    pub extension: String,
}
