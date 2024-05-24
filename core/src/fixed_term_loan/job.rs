use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{repo::*, state::*};
use crate::{job::*, ledger::*, primitives::FixedTermLoanId};

#[derive(Clone, Serialize, Deserialize)]
pub struct FixedTermLoanJobConfig {
    pub loan_id: FixedTermLoanId,
}

pub struct FixedTermLoanJobInitializer {
    ledger: Ledger,
    repo: FixedTermLoanRepo,
}

impl FixedTermLoanJobInitializer {
    pub fn new(ledger: &Ledger, repo: FixedTermLoanRepo) -> Self {
        Self {
            ledger: ledger.clone(),
            repo,
        }
    }
}

const FIXED_TERM_LOAN_JOB: JobType = JobType::new("FixedTermLoanJob");
impl JobInitializer for FixedTermLoanJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        FIXED_TERM_LOAN_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(FixedTermLoanJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

pub struct FixedTermLoanJobRunner {
    config: FixedTermLoanJobConfig,
    repo: FixedTermLoanRepo,
    ledger: Ledger,
}

#[async_trait]
impl JobRunner for FixedTermLoanJobRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut loan = self.repo.find_by_id(self.config.loan_id).await?;
        match loan.state {
            FixedTermLoanState::Initializing => {
                let loan_id = self.ledger.create_accounts_for_loan(loan.id).await?;
                loan.set_ledger_account_id(loan_id)?;
                self.repo.persist(&mut loan).await?;
                return Ok(JobCompletion::Pause);
            }
            FixedTermLoanState::Collateralized => {
                // update USD allocation
            }
            _ => (),
        }
        Ok(JobCompletion::Complete)
    }
}
