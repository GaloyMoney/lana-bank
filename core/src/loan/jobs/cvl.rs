use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use crate::{
    audit::*,
    authorization::{LoanAction, LoanAllOrOne, Object},
    job::*,
    loan::{repo::*, terms::CVLPct, LoanByCollateralizationRatioCursor, Subject, SystemNode},
    price::Price,
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
    audit: Audit,
    price: Price,
}

impl LoanProcessingJobInitializer {
    pub fn new(repo: LoanRepo, price: &Price, audit: &Audit) -> Self {
        Self {
            repo,
            price: price.clone(),
            audit: audit.clone(),
        }
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
            price: self.price.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct LoanProcessingJobRunner {
    config: LoanJobConfig,
    repo: LoanRepo,
    price: Price,
    audit: Audit,
}

#[async_trait]
impl JobRunner for LoanProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = self.price.usd_cents_per_btc().await?;
        let mut has_next_page = true;
        let mut after: Option<LoanByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut loans = self
                .repo
                .list_by_collateralization_ratio(crate::query::PaginatedQueryArgs::<
                    LoanByCollateralizationRatioCursor,
                > {
                    first: 10,
                    after,
                })
                .await?;
            (after, has_next_page) = (loans.end_cursor, loans.has_next_page);
            let mut db = current_job.pool().begin().await?;
            let audit_info = self
                .audit
                .record_entry_in_tx(
                    &mut db,
                    &Subject::System(SystemNode::Core),
                    Object::Loan(LoanAllOrOne::All),
                    LoanAction::UpdateCollateralizationState,
                    true,
                )
                .await?;

            let mut at_least_one = false;

            for loan in loans.entities.iter_mut() {
                if loan
                    .maybe_update_collateralization(
                        price,
                        self.config.upgrade_buffer_cvl_pct,
                        audit_info,
                    )
                    .is_some()
                {
                    self.repo.persist_in_tx(&mut db, loan).await?;
                    at_least_one = true;
                }
            }

            if at_least_one {
                db.commit().await?;
            } else {
                break;
            }
        }

        Ok(JobCompletion::RescheduleAt(
            chrono::Utc::now() + self.config.job_interval,
        ))
    }
}
