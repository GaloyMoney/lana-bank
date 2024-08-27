use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use crate::{
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
}

impl LoanProcessingJobInitializer {
    pub fn new(repo: LoanRepo) -> Self {
        Self { repo }
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
        }))
    }
}

pub struct LoanProcessingJobRunner {
    config: LoanJobConfig,
    repo: LoanRepo,
}

#[async_trait]
impl JobRunner for LoanProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // TODO: Add price service call here (consider checking for stale)
        let price = PriceOfOneBTC::new(UsdCents::from(5000000));

        let mut db_tx = current_job.pool().begin().await?;

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
                    self.repo.persist_in_tx(&mut db_tx, loan).await?;
                }
            }
        }

        Ok(JobCompletion::RescheduleAtWithTx(
            db_tx,
            chrono::Utc::now() + self.config.job_interval,
        ))
    }
}
