use async_trait::async_trait;
use credit_facility::{CreditFacilityInterestAccrual, CreditFacilityInterestIncurrence};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    audit::*,
    authorization::{CreditFacilityAction, Object},
    credit_facility::{repo::*, CreditFacilityError},
    job::*,
    ledger::*,
    primitives::CreditFacilityId,
    terms::InterestPeriod,
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

#[derive(Clone)]
struct ConfirmedIncurrence {
    incurrence: CreditFacilityInterestIncurrence,
    accrual: Option<CreditFacilityInterestAccrual>,
    next_period: Option<InterestPeriod>,
}

pub struct CreditFacilityProcessingJobRunner {
    config: CreditFacilityJobConfig,
    credit_facility_repo: CreditFacilityRepo,
    ledger: Ledger,
    audit: Audit,
}

impl CreditFacilityProcessingJobRunner {
    #[es_entity::retry_on_concurrent_modification]
    async fn confirm_interest_incurrence_and_accrual(
        &self,
        db: &mut es_entity::DbOp<'_>,
        audit_info: &AuditInfo,
    ) -> Result<ConfirmedIncurrence, CreditFacilityError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let confirmed_incurrence = {
            let outstanding = credit_facility.outstanding();
            let account_ids = credit_facility.account_ids;

            let accrual = credit_facility
                .interest_accrual_in_progress()
                .expect("Accrual in progress should exist for scheduled job");

            let interest_incurrence = accrual.initiate_incurrence(outstanding, account_ids);
            let interest_accrual =
                accrual.confirm_incurrence(interest_incurrence.clone(), audit_info.clone());

            ConfirmedIncurrence {
                incurrence: interest_incurrence,
                accrual: interest_accrual,
                next_period: accrual.next_incurrence_period(),
            }
        };

        if let Some(interest_accrual) = confirmed_incurrence.clone().accrual {
            credit_facility.confirm_interest_accrual(interest_accrual, audit_info.clone());
        }

        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        Ok(confirmed_incurrence)
    }
}

#[async_trait]
impl JobRunner for CreditFacilityProcessingJobRunner {
    #[instrument(
        name = "credit-facility.interest.job",
        skip(self, current_job),
        fields(attempt)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let span = tracing::Span::current();
        span.record("attempt", current_job.attempt());

        let mut db = self.credit_facility_repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                Object::CreditFacility,
                CreditFacilityAction::RecordInterest,
            )
            .await?;

        let ConfirmedIncurrence {
            incurrence: interest_incurrence,
            accrual: interest_accrual,
            next_period: next_incurrence_period,
        } = self
            .confirm_interest_incurrence_and_accrual(&mut db, &audit_info)
            .await?;
        self.ledger
            .record_credit_facility_interest_incurrence(interest_incurrence)
            .await?;
        if let Some(interest_accrual) = interest_accrual {
            self.ledger
                .record_credit_facility_interest_accrual(interest_accrual)
                .await?;
        }
        if let Some(period) = next_incurrence_period {
            return Ok(JobCompletion::RescheduleAtWithOp(db, period.end));
        }

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_tx(db.tx(), self.config.credit_facility_id)
            .await?;
        if let Some(period) = credit_facility.start_interest_accrual(audit_info)? {
            self.credit_facility_repo
                .update_in_op(&mut db, &mut credit_facility)
                .await?;

            return Ok(JobCompletion::RescheduleAtWithOp(db, period.end));
        }

        println!(
            "Credit Facility interest job completed for credit_facility: {:?}",
            self.config.credit_facility_id
        );
        Ok(JobCompletion::CompleteWithOp(db))
    }
}
