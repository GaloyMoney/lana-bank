use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use crate::{
    data_export::Export,
    job::*,
    loan::{repo::*, terms::CVLPct, LoanCursor},
    primitives::{PriceOfOneBTC, UsdCents},
};

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct LoanJobConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub job_interval: Duration,
    pub upgrade_buffer_cvl_pct: CVLPct,
}

pub struct LoanProcessingJobInitializer {
    repo: LoanRepo,
    export: Export,
}

impl LoanProcessingJobInitializer {
    pub fn new(repo: LoanRepo, export: Export) -> Self {
        Self { repo, export }
    }
}

const LOAN_CVL_PROCESSING_JOB: JobType = JobType::new("loan-cvl-processing");
impl JobInitializer for LoanProcessingJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        LOAN_CVL_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(LoanProcessingJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            export: self.export.clone(),
        }))
    }
}

pub struct LoanProcessingJobRunner {
    config: LoanJobConfig,
    repo: LoanRepo,
    export: Export,
}

#[async_trait]
impl JobRunner for LoanProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // TODO: Add price service call here (consider checking for stale)
        let price = PriceOfOneBTC::new(UsdCents::from(5000000));

        let mut has_next_page = true;
        let mut after: Option<LoanCursor> = None;
        while has_next_page {
            let mut loans = self
                .repo
                .list(crate::query::PaginatedQueryArgs::<LoanCursor> { first: 100, after })
                .await?;
            (after, has_next_page) = (loans.end_cursor, loans.has_next_page);

            for loan in loans.entities.iter_mut() {
                if loan
                    .maybe_update_collateralization(price, self.config.upgrade_buffer_cvl_pct)
                    .is_some()
                {
                    let mut db_tx = current_job.pool().begin().await?;

                    let n_events = self.repo.persist_in_tx(&mut db_tx, loan).await?;
                    self.export
                        .export_last(
                            &mut db_tx,
                            crate::loan::BQ_TABLE_NAME,
                            n_events,
                            &loan.events,
                        )
                        .await?;
                    db_tx.commit().await?;
                }
            }
        }

        Ok(JobCompletion::RescheduleAt(
            chrono::Utc::now() + self.config.job_interval,
        ))
    }
}
