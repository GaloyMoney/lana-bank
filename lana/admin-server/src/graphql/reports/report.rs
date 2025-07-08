use async_graphql::*;

use crate::primitives::*;

use super::super::loader::LanaDataLoader;

pub use lana_app::report::{File, Report as DomainReport};

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
    pub(super) entity: Arc<DomainReport>,
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
    path_in_bucket: String,
}

impl From<File> for ReportFile {
    fn from(file: File) -> Self {
        ReportFile {
            extension: file.extension,
            path_in_bucket: file.path_in_bucket,
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

    async fn report_run(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<super::report_run::ReportRun> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let report_run = loader
            .load_one(self.entity.run_id)
            .await?
            .expect("report run not found");
        Ok(report_run)
    }
}

#[derive(InputObject)]
pub struct ReportCreateInput {
    pub external_id: String,
    pub run_id: UUID,
    pub name: String,
    pub norm: String,
    pub files: Vec<ReportFileInput>,
}

#[derive(InputObject)]
pub struct ReportFileInput {
    pub extension: String,
    pub path_in_bucket: String,
}

impl From<ReportFileInput> for File {
    fn from(input: ReportFileInput) -> Self {
        File {
            extension: input.extension,
            path_in_bucket: input.path_in_bucket,
        }
    }
}

crate::mutation_payload! { ReportCreatePayload, report: Report }
