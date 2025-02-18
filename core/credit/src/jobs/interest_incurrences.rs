use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    audit::*,
    authorization::{CreditFacilityAction, Object},
    credit_facility::{ledger::*, repo::*, CreditFacilityError, CreditFacilityInterestIncurrence},
    job::*,
    primitives::{CreditFacilityId, InterestAccrualIdx},
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
    ledger: CreditLedger,
    credit_facility_repo: CreditFacilityRepo,
    audit: Audit,
    jobs: Jobs,
}

impl CreditFacilityProcessingJobInitializer {
    pub fn new(
        ledger: &CreditLedger,
        credit_facility_repo: CreditFacilityRepo,
        audit: &Audit,
        jobs: &Jobs,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            credit_facility_repo,
            audit: audit.clone(),
            jobs: jobs.clone(),
        }
    }
}

const CREDIT_FACILITY_INTEREST_INCURRENCE_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-interest-incurrence-processing");
impl JobInitializer for CreditFacilityProcessingJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_INTEREST_INCURRENCE_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            audit: self.audit.clone(),
            jobs: self.jobs.clone(),
        }))
    }
}

#[derive(Clone)]
struct ConfirmedIncurrence {
    incurrence: CreditFacilityInterestIncurrence,
    next_period: Option<InterestPeriod>,
    accrual_idx: InterestAccrualIdx,
}

pub struct CreditFacilityProcessingJobRunner {
    config: CreditFacilityJobConfig,
    credit_facility_repo: CreditFacilityRepo,
    ledger: CreditLedger,
    audit: Audit,
    jobs: Jobs,
}

impl CreditFacilityProcessingJobRunner {
    #[es_entity::retry_on_concurrent_modification]
    async fn confirm_interest_incurrence(
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

            let interest_incurrence = accrual.record_incurrence(outstanding, audit_info.clone());

            ConfirmedIncurrence {
                incurrence: (interest_incurrence, account_ids).into(),
                next_period: accrual.next_incurrence_period(),
                accrual_idx: accrual.idx,
            }
        };

        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        Ok(confirmed_incurrence)
    }
}

#[async_trait]
impl JobRunner for CreditFacilityProcessingJobRunner {
    #[instrument(
        name = "credit-facility.interest-incurrences.job",
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
            next_period: next_incurrence_period,
            accrual_idx,
        } = self
            .confirm_interest_incurrence(&mut db, &audit_info)
            .await?;

        let (now, mut tx) = (db.now(), db.into_tx());
        let sub_op = {
            use sqlx::Acquire;
            es_entity::DbOp::new(tx.begin().await?, now)
        };
        self.ledger
            .record_interest_incurrence(sub_op, interest_incurrence)
            .await?;

        let mut db = es_entity::DbOp::new(tx, now);
        if let Some(period) = next_incurrence_period {
            Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
        } else {
            self.jobs
                .create_and_spawn_in_op(
                    &mut db,
                    uuid::Uuid::new_v4(),
                    super::interest_accruals::CreditFacilityJobConfig {
                        credit_facility_id: self.config.credit_facility_id,
                    },
                )
                .await?;
            println!(
            "Credit Facility interest incurrences job completed for accrual index {:?} for credit_facility: {:?}",
            accrual_idx, self.config.credit_facility_id
        );
            Ok(JobCompletion::CompleteWithOp(db))
        }
    }
}
