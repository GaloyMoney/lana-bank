use async_trait::async_trait;
use core_money::UsdCents;
use serde::{Deserialize, Serialize};

use crate::{
    audit::*,
    authorization::{CreditFacilityAction, Object},
    credit_facility::repo::*,
    job::*,
    ledger::*,
    primitives::CreditFacilityId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig {
    pub credit_facility_id: CreditFacilityId,
}
impl JobConfig for CreditFacilityJobConfig {
    type Initializer = CreditFacilityProcessingJobInitializer;
}

pub struct CreditFacilityProcessingJobInitializer {
    ledger: Ledger,
    credit_facility_repo: CreditFacilityRepo,
    audit: Audit,
}

impl CreditFacilityProcessingJobInitializer {
    pub fn new(ledger: &Ledger, credit_facility_repo: CreditFacilityRepo, audit: &Audit) -> Self {
        Self {
            ledger: ledger.clone(),
            credit_facility_repo,
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_INTEREST_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-interest-processing");
impl JobInitializer for CreditFacilityProcessingJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_INTEREST_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            audit: self.audit.clone(),
        }))
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct CreditFacilityProcessingJobData {
    sequence: i64,
    msg: String,
    external_id: String,
    updated_true: i64,
    updated_e: String,
    count_incurred: usize,
    total_incurred: UsdCents,
    branch_1: i64,
    branch_1a: i64,
    branch_2: i64,
    branch_3: i64,
    step_1: i64,
    step_1a: i64,
    step_1b: i64,
    step_1c: i64,
    step_1d: i64,
    step_2: i64,
}

pub struct CreditFacilityProcessingJobRunner {
    config: CreditFacilityJobConfig,
    credit_facility_repo: CreditFacilityRepo,
    ledger: Ledger,
    audit: Audit,
}

#[async_trait]
impl JobRunner for CreditFacilityProcessingJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityProcessingJobData>()?
            .unwrap_or_default();
        state.sequence += 1;
        current_job.update_execution_state(state.clone()).await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let mut db = self.credit_facility_repo.begin_op().await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                Object::CreditFacility,
                CreditFacilityAction::RecordInterest,
            )
            .await?;

        state.step_1 += 1;
        current_job.update_execution_state(state.clone()).await?;

        let (interest_accrual, next_incurrence_period) = {
            let outstanding = credit_facility.outstanding();
            let account_ids = credit_facility.account_ids;

            let accrual = credit_facility
                .interest_accrual_in_progress()
                .expect("Accrual in progress should exist for scheduled job");

            let interest_incurrence = accrual.initiate_incurrence(outstanding, account_ids);

            state.step_1a += 1;
            state.external_id = interest_incurrence.clone().tx_ref;
            current_job.update_execution_state(state.clone()).await?;

            match self
                .ledger
                .record_credit_facility_interest_incurrence(interest_incurrence.clone())
                .await
            {
                Err(e) => {
                    state.step_1b += 1;
                    state.msg = e.to_string();
                    current_job.update_execution_state(state.clone()).await?;
                    return Err(Box::new(e));
                }
                Ok(_) => {
                    state.step_1c += 1;
                    current_job.update_execution_state(state.clone()).await?;
                    ()
                }
            }

            state.step_1d += 1;
            current_job.update_execution_state(state.clone()).await?;

            (
                accrual.confirm_incurrence(interest_incurrence, audit_info.clone()),
                accrual.next_incurrence_period(),
            )
        };

        state.step_2 += 1;
        current_job.update_execution_state(state.clone()).await?;

        if let Some(interest_accrual) = interest_accrual {
            self.ledger
                .record_credit_facility_interest_accrual(interest_accrual.clone())
                .await?;
            credit_facility.confirm_interest_accrual(interest_accrual, audit_info.clone());
        }

        if let Some(period) = next_incurrence_period {
            state.branch_1 += 1;
            current_job.update_execution_state(state.clone()).await?;

            {
                let accrual = credit_facility
                    .interest_accrual_in_progress()
                    .expect("Accrual in progress should exist for scheduled job");
                state.total_incurred = accrual.total_incurred();
                state.count_incurred = accrual.count_incurred();
                current_job.update_execution_state(state.clone()).await?;
            }

            match self
                .credit_facility_repo
                .update_in_op(&mut db, &mut credit_facility)
                .await
            {
                Ok(_) => {
                    state.updated_true += 1;
                    current_job.update_execution_state(state.clone()).await?;
                }
                Err(e) => {
                    state.updated_e = e.to_string();
                    current_job.update_execution_state(state.clone()).await?;
                    return Err(Box::new(e));
                }
            };

            state.branch_1a += 1;
            current_job.update_execution_state(state.clone()).await?;
            Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
        } else if let Some(period) = credit_facility.start_interest_accrual(audit_info)? {
            state.branch_2 += 1;
            current_job.update_execution_state(state.clone()).await?;

            self.credit_facility_repo
                .update_in_op(&mut db, &mut credit_facility)
                .await?;

            Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
        } else {
            state.branch_3 += 1;
            current_job.update_execution_state(state.clone()).await?;

            self.credit_facility_repo
                .update_in_op(&mut db, &mut credit_facility)
                .await?;
            println!(
                "Credit Facility interest job completed for credit_facility: {:?}",
                credit_facility.id
            );

            Ok(JobCompletion::CompleteWithOp(db))
        }
    }
}
