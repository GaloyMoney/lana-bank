use async_trait::async_trait;
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
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
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

        let id = credit_facility
            .interest_accrual_in_progress()
            .expect("Accrual in progress should exist for scheduled job");
        let mut accrual = self.interest_accrual_repo.find_by_id(id).await?;

        let interest_incurrence =
            accrual.initiate_incurrence(credit_facility.outstanding(), credit_facility.account_ids);

        self.ledger
            .record_credit_facility_interest_incurrence(interest_incurrence.clone())
            .await?;
        let interest_accrual = accrual.confirm_incurrence(interest_incurrence, audit_info.clone());

        if let Some(interest_accrual) = interest_accrual {
            self.ledger
                .record_credit_facility_interest_accrual(interest_accrual.clone())
                .await?;
            accrual.confirm_accrual(interest_accrual.clone(), audit_info.clone());

            credit_facility.confirm_interest_accrual(
                interest_accrual,
                accrual.idx,
                audit_info.clone(),
            );
            self.credit_facility_repo
                .update_in_op(&mut db, &mut credit_facility)
                .await?;
        }

        self.interest_accrual_repo
            .update_in_op(&mut db, &mut accrual)
            .await?;

        if let Some(period) = accrual.next_incurrence_period() {
            Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
        } else if let Some(new_accrual) = credit_facility.start_interest_accrual(audit_info)? {
            self.credit_facility_repo
                .update_in_op(&mut db, &mut credit_facility)
                .await?;
            let new_incurrence_period = self
                .interest_accrual_repo
                .create_in_op(&mut db, new_accrual)
                .await?
                .next_incurrence_period()
                .expect("New accrual should have first incurrence period");

            Ok(JobCompletion::RescheduleAtWithOp(
                db,
                new_incurrence_period.end,
            ))
        } else {
            println!(
                "Credit Facility interest job completed for credit_facility: {:?}",
                credit_facility.id
            );

            Ok(JobCompletion::CompleteWithOp(db))
        }
    }
}
