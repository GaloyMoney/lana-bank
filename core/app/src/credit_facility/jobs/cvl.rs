use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use crate::{
    audit::*,
    authorization::{CreditFacilityAction, Object},
    credit_facility::{repo::*, CreditFacilityByCollateralizationRatioCursor, Subject},
    job::*,
    price::Price,
    terms::CVLPct,
};

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub job_interval: Duration,
    pub upgrade_buffer_cvl_pct: CVLPct,
}

pub struct CreditFacilityProcessingJobInitializer {
    repo: CreditFacilityRepo,
    audit: Audit,
    price: Price,
}

impl CreditFacilityProcessingJobInitializer {
    pub fn new(repo: CreditFacilityRepo, price: &Price, audit: &Audit) -> Self {
        Self {
            repo,
            price: price.clone(),
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_CVL_PROCESSING_JOB: JobType = JobType::new("credit-facility-cvl-processing");
impl JobInitializer for CreditFacilityProcessingJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_CVL_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner {
            config: job.data()?,
            repo: self.repo.clone(),
            price: self.price.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct CreditFacilityProcessingJobRunner {
    config: CreditFacilityJobConfig,
    repo: CreditFacilityRepo,
    price: Price,
    audit: Audit,
}

#[async_trait]
impl JobRunner for CreditFacilityProcessingJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = self.price.usd_cents_per_btc().await?;
        let mut has_next_page = true;
        let mut after: Option<CreditFacilityByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut credit_facilities = self
                .repo
                .list_by_collateralization_ratio(es_entity::PaginatedQueryArgs::<
                    CreditFacilityByCollateralizationRatioCursor,
                > {
                    first: 10,
                    after,
                })
                .await?;
            (after, has_next_page) = (
                credit_facilities.end_cursor,
                credit_facilities.has_next_page,
            );
            let mut db = current_job.pool().begin().await?;
            let audit_info = self
                .audit
                .record_entry(
                    &Subject::core(),
                    Object::CreditFacility,
                    CreditFacilityAction::UpdateCollateralizationState,
                    true,
                )
                .await?;

            let mut at_least_one = false;

            for facility in credit_facilities.entities.iter_mut() {
                if facility
                    .maybe_update_collateralization(
                        price,
                        self.config.upgrade_buffer_cvl_pct,
                        &audit_info,
                    )
                    .is_some()
                {
                    self.repo.update_in_tx(&mut db, facility).await?;
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
