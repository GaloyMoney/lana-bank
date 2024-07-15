use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{error::LoanError, repo::*};
use crate::{
    job::*,
    ledger::*,
    primitives::{LedgerTxId, LoanId},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct LoanJobConfig {
    pub loan_id: LoanId,
}

pub struct LoanProcessingJobInitializer {
    ledger: Ledger,
    repo: LoanRepo,
}

impl LoanProcessingJobInitializer {
    pub fn new(ledger: &Ledger, repo: LoanRepo) -> Self {
        Self {
            ledger: ledger.clone(),
            repo,
        }
    }
}

const LOAN_PROCESSING_JOB: JobType = JobType::new("loan-processing");
impl JobInitializer for LoanProcessingJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        LOAN_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(LoanProcessingJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

pub struct LoanProcessingJobRunner {
    config: LoanJobConfig,
    repo: LoanRepo,
    ledger: Ledger,
}

#[async_trait]
impl JobRunner for LoanProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut loan = self.repo.find_by_id(self.config.loan_id).await?;
        if !loan.is_collateralized() {
            let mut tx = current_job.pool().begin().await?;
            let tx_id = LedgerTxId::new();
            self.ledger
                .collateralize_loan(
                    tx_id,
                    loan.account_ids,
                    loan.user_account_ids,
                    loan.initial_collateral(),
                    loan.initial_principal(),
                    format!("{}-approval", loan.id),
                )
                .await?;
            loan.collateralize(tx_id);
            self.repo.persist_in_tx(&mut tx, &mut loan).await?;
            return Ok(JobCompletion::RescheduleAtWithTx(
                tx,
                loan.next_interest_at().expect("first interest"),
            ));
        }

        let tx_id = LedgerTxId::new();
        let (interest, tx_ref) = match loan.add_interest(tx_id) {
            Err(LoanError::AlreadyCompleted) => {
                return Ok(JobCompletion::Complete);
            }
            Ok(tx_ref) => tx_ref,
            Err(_) => unreachable!(),
        };

        let penalty = match loan.add_penalty(tx_id) {
            Err(LoanError::AlreadyCompleted) => {
                return Ok(JobCompletion::Complete);
            }
            Ok(tx_ref) => tx_ref,
            Err(_) => unreachable!(),
        };

        let mut db_tx = current_job.pool().begin().await?;
        self.repo.persist_in_tx(&mut db_tx, &mut loan).await?;

        self.ledger
            .record_loan_interest(tx_id, loan.account_ids, tx_ref, interest, penalty)
            .await?;

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
