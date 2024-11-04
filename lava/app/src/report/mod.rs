mod config;
pub mod dataform_client;
mod entity;
pub mod error;
mod jobs;
mod repo;
pub mod upload;

use authz::PermissionCheck;
use tracing::instrument;

use crate::{
    authorization::{Authorization, Object, ReportAction},
    data_export::Export,
    job::Jobs,
    primitives::{ReportId, Subject},
    storage::Storage,
};

pub use config::*;
pub use entity::*;
use error::*;
use jobs as report_jobs;
use repo::*;

#[derive(Clone)]
pub struct Reports {
    pool: sqlx::PgPool,
    authz: Authorization,
    repo: ReportRepo,
    jobs: Jobs,
    storage: Storage,
}

impl Reports {
    pub async fn init(
        pool: &sqlx::PgPool,
        config: &ReportConfig,
        authz: &Authorization,
        jobs: &Jobs,
        storage: &Storage,
        export: &Export,
    ) -> Result<Self, ReportError> {
        let repo = ReportRepo::new(pool, export);
        jobs.add_initializer(report_jobs::generate::GenerateReportInitializer::new(
            &repo,
            config,
            authz.audit(),
            storage,
        ));
        jobs.add_initializer_and_spawn_unique(
            report_jobs::create::CreateReportInitializer::new(&repo, jobs, authz.audit()),
            report_jobs::create::CreateReportJobConfig {
                job_interval: report_jobs::create::CreateReportInterval::EndOfDay,
            },
        )
        .await?;

        Ok(Self {
            repo,
            pool: pool.clone(),
            authz: authz.clone(),
            jobs: jobs.clone(),
            storage: storage.clone(),
        })
    }

    #[instrument(name = "report.create", skip(self))]
    pub async fn create(&self, sub: &Subject) -> Result<Report, ReportError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::Report, ReportAction::Create)
            .await?;

        let new_report = NewReport::builder()
            .id(ReportId::new())
            .audit_info(audit_info)
            .build()
            .expect("Could not build report");

        let mut db = self.pool.begin().await?;
        let report = self.repo.create_in_tx(&mut db, new_report).await?;
        self.jobs
            .create_and_spawn_in_tx(
                &mut db,
                report.id,
                report_jobs::generate::GenerateReportConfig {
                    report_id: report.id,
                },
            )
            .await?;
        db.commit().await?;
        Ok(report)
    }

    #[instrument(name = "report.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: impl Into<ReportId> + std::fmt::Debug,
    ) -> Result<Option<Report>, ReportError> {
        self.authz
            .enforce_permission(sub, Object::Report, ReportAction::Read)
            .await?;

        match self.repo.find_by_id(id.into()).await {
            Ok(report) => Ok(Some(report)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_reports(&self, sub: &Subject) -> Result<Vec<Report>, ReportError> {
        self.authz
            .enforce_permission(sub, Object::Report, ReportAction::List)
            .await?;

        Ok(self
            .repo
            .list_by_created_at(Default::default(), es_entity::ListDirection::Ascending)
            .await?
            .entities)
    }

    pub async fn generate_download_links(
        &self,
        sub: &Subject,
        report_id: ReportId,
    ) -> Result<GeneratedReportDownloadLinks, ReportError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::Report, ReportAction::GenerateDownloadLink)
            .await?;

        let mut report = self.repo.find_by_id(report_id).await?;

        let mut db_tx = self.pool.begin().await?;

        let mut download_links = vec![];
        for location in report.download_links() {
            let url = self.storage.generate_download_link(&location).await?;

            download_links.push(ReportDownloadLink {
                report_name: location.report_name.clone(),
                url,
            });

            report.download_link_generated(audit_info.clone(), location);
        }

        self.repo.update_in_tx(&mut db_tx, &mut report).await?;
        Ok(GeneratedReportDownloadLinks {
            report_id,
            links: download_links,
        })
    }
}
