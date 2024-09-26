mod cloud_storage;
mod config;
pub mod dataform_client;
mod entity;
pub mod error;
mod jobs;
mod repo;
pub mod upload;

use crate::{
    audit::*,
    authorization::{Authorization, Object, ReportAction},
    constants::CREATE_REPORT_JOB_ID,
    entity::EntityError,
    job::Jobs,
    primitives::{ReportId, Subject},
};

use cloud_storage::generate_download_link;
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
        jobs.add_initializer(report_jobs::generate::GenerateReportInitializer::new(
            &repo, config, audit,
        ));
        jobs.add_initializer(report_jobs::create::CreateReportInitializer::new(
            &repo, jobs, audit,
        ));

        Self {
            repo,
            pool: pool.clone(),
            authz: authz.clone(),
            jobs: jobs.clone(),
        }
    }

    pub async fn spawn_global_jobs(&self) -> Result<(), ReportError> {
        let mut db_tx = self.pool.begin().await?;
        match self
            .jobs
            .create_and_spawn_job::<report_jobs::create::CreateReportInitializer, _>(
                &mut db_tx,
                CREATE_REPORT_JOB_ID,
                "create-report-job".to_string(),
                report_jobs::create::CreateReportJobConfig {
                    job_interval: report_jobs::create::CreateReportInterval::EndOfDay,
                },
            )
            .await
        {
            Err(crate::job::error::JobError::DuplicateId) => (),
            Err(e) => return Err(e.into()),
            _ => (),
        }
        db_tx.commit().await?;
        Ok(())
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
            .create_and_spawn_job::<report_jobs::generate::GenerateReportInitializer, _>(
                &mut db,
                report.id,
                "generate_report".to_string(),
                report_jobs::generate::GenerateReportConfig {
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
            let url = generate_download_link(&location).await?;

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
