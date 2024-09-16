mod config;
pub mod dataform_client;
mod entity;
pub mod error;
mod job;
mod repo;

use crate::{
    authorization::{Authorization, Object, ReportAction},
    job::Jobs,
    primitives::Subject,
};

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
}

impl Reports {
    pub fn new(
        pool: &sqlx::PgPool,
        config: &ReportConfig,
        authz: &Authorization,
        jobs: &Jobs,
    ) -> Self {
        let repo = ReportRepo::new(pool);
        jobs.add_initializer(job::GenerateReportInitializer::new(&repo, config));
        Self {
            repo,
            pool: pool.clone(),
            authz: authz.clone(),
            jobs: jobs.clone(),
        }
    }

    pub async fn create(&self, sub: &Subject) -> Result<Report, ReportError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Report, ReportAction::Create)
            .await?;

        let new_report = NewReport::builder()
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
}
