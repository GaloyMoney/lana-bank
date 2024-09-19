mod cloud_storage;
mod config;
pub mod dataform_client;
mod entity;
pub mod error;
mod job;
mod repo;
pub mod upload;

use crate::{
    audit::*,
    authorization::{Authorization, Object, ReportAction},
    entity::EntityError,
    job::Jobs,
    primitives::{ReportId, Subject},
};

use cloud_storage::generate_download_link;
pub use config::*;
pub use entity::*;
use error::*;
use repo::*;

#[derive(Clone)]
pub struct Reports {
    pool: sqlx::PgPool,
    authz: Authorization,
    repo: ReportRepo,
    jobs: Jobs,
    config: ReportConfig,
}

impl Reports {
    pub fn new(
        pool: &sqlx::PgPool,
        config: &ReportConfig,
        authz: &Authorization,
        audit: &Audit,
        jobs: &Jobs,
    ) -> Self {
        let repo = ReportRepo::new(pool);
        jobs.add_initializer(job::GenerateReportInitializer::new(&repo, config, audit));

        Self {
            repo,
            pool: pool.clone(),
            authz: authz.clone(),
            jobs: jobs.clone(),
            config: config.clone(),
        }
    }

    pub async fn create(&self, sub: &Subject) -> Result<Report, ReportError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Report, ReportAction::Create)
            .await?;

        let new_report = NewReport::builder()
            .id(ReportId::new())
            .audit_info(audit_info)
            .build()
            .expect("Could not build report");

        let mut db = self.pool.begin().await?;
        let report = self.repo.create_in_tx(&mut db, new_report).await?;
        self.jobs
            .create_and_spawn_job::<job::GenerateReportInitializer, _>(
                &mut db,
                report.id,
                "generate_report".to_string(),
                job::GenerateReportConfig {
                    report_id: report.id,
                },
            )
            .await?;
        db.commit().await?;
        Ok(report)
    }

    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: ReportId,
    ) -> Result<Option<Report>, ReportError> {
        self.authz
            .check_permission(sub, Object::Report, ReportAction::Read)
            .await?;

        match self.repo.find_by_id(id).await {
            Ok(report) => Ok(Some(report)),
            Err(ReportError::EntityError(EntityError::NoEntityEventsPresent)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_reports(&self, sub: &Subject) -> Result<Vec<Report>, ReportError> {
        self.authz
            .check_permission(sub, Object::Report, ReportAction::List)
            .await?;
        self.repo.list().await
    }

    pub async fn generate_download_links(
        &self,
        sub: &Subject,
        report_id: ReportId,
    ) -> Result<GeneratedReportDownloadLinks, ReportError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Report, ReportAction::GenerateDownloadLink)
            .await?;

        let mut report = self.repo.find_by_id(report_id).await?;

        let mut db_tx = self.pool.begin().await?;

        let mut download_links = vec![];
        for location in report.download_links() {
            let url = generate_download_link(&location, self.config.download_link_duration).await?;

            download_links.push(ReportDownloadLink {
                report_name: location.report_name.clone(),
                url,
            });

            report.download_link_generated(audit_info, location);
        }

        self.repo.persist_in_tx(&mut db_tx, &mut report).await?;
        Ok(GeneratedReportDownloadLinks {
            report_id,
            links: download_links,
        })
    }
}
