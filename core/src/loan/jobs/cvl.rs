use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{repo::*, CLVJobInterval, CVLPct, LoanCursor, PriceOfOneBTC, UsdCents};
use crate::job::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoanJobConfig {
    pub job_interval: CLVJobInterval,
    pub collateral_upgrade_buffer: CVLPct,
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
                    .maybe_update_collateralization(price, self.config.collateral_upgrade_buffer)
                    .is_some()
                {
                    self.repo.persist_in_tx(&mut db_tx, loan).await?;
                }
            }
        }

        Ok(JobCompletion::RescheduleAtWithTx(
            db_tx,
            self.config.job_interval.add_to_time(chrono::Utc::now()),
        ))
    }
}
