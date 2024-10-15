use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    audit::*,
    authorization::{CreditFacilityAction, Object},
    credit_facility::{repo::*, InterestAccrualRepo, Subject},
    job::*,
    ledger::*,
    primitives::{CreditFacilityId, SystemNode},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig {
    pub credit_facility_id: CreditFacilityId,
}

pub struct CreditFacilityProcessingJobInitializer {
    ledger: Ledger,
    credit_facility_repo: CreditFacilityRepo,
    interest_accrual_repo: InterestAccrualRepo,
    audit: Audit,
}

impl CreditFacilityProcessingJobInitializer {
    pub fn new(
        ledger: &Ledger,
        credit_facility_repo: CreditFacilityRepo,
        interest_accrual_repo: InterestAccrualRepo,
        audit: &Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            credit_facility_repo,
            interest_accrual_repo,
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
            config: job.data()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            interest_accrual_repo: self.interest_accrual_repo.clone(),
            ledger: self.ledger.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct CreditFacilityProcessingJobRunner {
    config: CreditFacilityJobConfig,
    credit_facility_repo: CreditFacilityRepo,
    interest_accrual_repo: InterestAccrualRepo,
    ledger: Ledger,
    audit: Audit,
}

#[async_trait]
impl JobRunner for CreditFacilityProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let mut db_tx = current_job.pool().begin().await?;

        let audit_info = self
            .audit
            .record_entry_in_tx(
                &mut db_tx,
                &Subject::System(SystemNode::Core),
                Object::CreditFacility,
                CreditFacilityAction::RecordInterest,
                true,
            )
            .await?;

        let idx = credit_facility
            .interest_accrual_in_progress()
            .expect("Accrual in progress should exist for scheduled job");
        let mut accrual = self
            .interest_accrual_repo
            .find_by_idx_for_credit_facility(credit_facility.id, idx)
            .await?;

        let interest_incurrence =
            accrual.initiate_incurrence(credit_facility.outstanding(), credit_facility.account_ids);

        self.ledger
            .record_credit_facility_interest_incurrence(interest_incurrence.clone())
            .await?;
        let interest_accrual = accrual.confirm_incurrence(
            interest_incurrence,
            credit_facility.account_ids,
            audit_info,
        );

        if let Some(interest_accrual) = interest_accrual {
            let executed_at = self
                .ledger
                .record_credit_facility_interest_accrual(interest_accrual.clone())
                .await?;
            accrual.confirm_accrual(interest_accrual.clone(), executed_at, audit_info);

            credit_facility.confirm_interest_accrual(
                interest_accrual,
                accrual.idx,
                executed_at,
                audit_info,
            );
            self.credit_facility_repo
                .persist_in_tx(&mut db_tx, &mut credit_facility)
                .await?;
        }

        self.interest_accrual_repo
            .persist_in_tx(&mut db_tx, &mut accrual)
            .await?;

        if let Some(period) = accrual.next_incurrence_period() {
            Ok(JobCompletion::RescheduleAtWithTx(db_tx, period.end))
        } else if let Some(new_accrual) = credit_facility.start_interest_accrual(audit_info)? {
            self.credit_facility_repo
                .persist_in_tx(&mut db_tx, &mut credit_facility)
                .await?;
            let new_accrual_period = self
                .interest_accrual_repo
                .create_in_tx(&mut db_tx, new_accrual)
                .await?
                .next_incurrence_period()
                .expect("New accrual should have first incurrence period");

            Ok(JobCompletion::RescheduleAtWithTx(
                db_tx,
                new_accrual_period.end,
            ))
        } else {
            println!(
                "Credit Facility interest job completed for credit_facility: {:?}",
                credit_facility.id
            );

            Ok(JobCompletion::CompleteWithTx(db_tx))
        }
    }
}
