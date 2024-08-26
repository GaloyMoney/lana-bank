use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    error::LoanError, repo::*, CLVJobInterval, CVLPct, LoanCursor, PriceOfOneBTC,
    StalePriceInterval, Subject, SystemNode, UsdCents,
};
use crate::{
    audit::*,
    authorization::{LoanAction, Object},
    job::*,
    ledger::*,
    primitives::LoanId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct LoanInterestJobConfig {
    pub loan_id: LoanId,
}

pub struct LoanInterestProcessingJobInitializer {
    ledger: Ledger,
    audit: Audit,
    repo: LoanRepo,
}

impl LoanInterestProcessingJobInitializer {
    pub fn new(ledger: &Ledger, repo: LoanRepo, audit: &Audit) -> Self {
        Self {
            ledger: ledger.clone(),
            repo,
            audit: audit.clone(),
        }
    }
}

const LOAN_INTEREST_PROCESSING_JOB: JobType = JobType::new("loan-interest-processing");
impl JobInitializer for LoanInterestProcessingJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        LOAN_INTEREST_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(LoanInterestProcessingJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct LoanInterestProcessingJobRunner {
    config: LoanInterestJobConfig,
    repo: LoanRepo,
    ledger: Ledger,
    audit: Audit,
}

#[async_trait]
impl JobRunner for LoanInterestProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut loan = self.repo.find_by_id(self.config.loan_id).await?;
        let mut db_tx = current_job.pool().begin().await?;
        let audit_info = self
            .audit
            .record_entry_in_tx(
                &mut db_tx,
                &Subject::System(SystemNode::Core),
                Object::Loan,
                LoanAction::RecordInterest,
                true,
            )
            .await?;
        let interest_accrual = match loan.initiate_interest() {
            Err(LoanError::AlreadyCompleted) => {
                return Ok(JobCompletion::Complete);
            }
            Ok(tx_ref) => tx_ref,
            Err(_) => unreachable!(),
        };

        let executed_at = self
            .ledger
            .record_loan_interest(interest_accrual.clone())
            .await?;

        loan.confirm_interest(interest_accrual, executed_at, audit_info);
        self.repo.persist_in_tx(&mut db_tx, &mut loan).await?;

        match loan.next_interest_at() {
            Some(next_interest_at) => {
                Ok(JobCompletion::RescheduleAtWithTx(db_tx, next_interest_at))
            }
            None => {
                println!("Loan interest job completed for loan: {:?}", loan.id);
                Ok(JobCompletion::CompleteWithTx(db_tx))
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoanCVLJobConfig {
    pub stale_price_interval: StalePriceInterval,
    pub job_interval: CLVJobInterval,
    pub collateral_upgrade_buffer: CVLPct,
}

pub struct LoanCVLProcessingJobInitializer {
    repo: LoanRepo,
}

impl LoanCVLProcessingJobInitializer {
    pub fn new(repo: LoanRepo) -> Self {
        Self { repo }
    }
}

const LOAN_CVL_PROCESSING_JOB: JobType = JobType::new("loan-cvl-processing");
impl JobInitializer for LoanCVLProcessingJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        LOAN_CVL_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(LoanCVLProcessingJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

pub struct LoanCVLProcessingJobRunner {
    config: LoanCVLJobConfig,
    repo: LoanRepo,
}

#[async_trait]
impl JobRunner for LoanCVLProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = PriceOfOneBTC::new(UsdCents::from(5000000), chrono::Utc::now());

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
                    .maybe_update_collateralization(price, self.config.collateral_upgrade_buffer)?
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
