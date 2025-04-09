use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo,
    error::CoreCreditError,
    obligation::{obligation_cursor::ObligationsByCreatedAtCursor, Obligation, ObligationRepo},
    obligation_aggregator::{ObligationAggregator, ObligationDataForAggregation},
    primitives::*,
    terms::CVLPct,
    CoreCreditAction, CoreCreditEvent, CoreCreditObject,
    CreditFacilitiesByCollateralizationRatioCursor,
};

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig<Perms, E> {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub job_interval: Duration,
    pub upgrade_buffer_cvl_pct: CVLPct,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for CreditFacilityJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = CreditFacilityProcessingJobInitializer<Perms, E>;
}
pub struct CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    obligation_repo: ObligationRepo<E>,
    credit_facility_repo: CreditFacilityRepo<E>,
    audit: Perms::Audit,
    price: Price,
}

impl<Perms, E> CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        obligation_repo: ObligationRepo<E>,
        credit_facility_repo: CreditFacilityRepo<E>,
        price: &Price,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            obligation_repo,
            credit_facility_repo,
            price: price.clone(),
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_CVL_PROCESSING_JOB: JobType = JobType::new("credit-facility-cvl-processing");
impl<Perms, E> JobInitializer for CreditFacilityProcessingJobInitializer<Perms, E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_CVL_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms, E> {
            config: job.config()?,
            obligation_repo: self.obligation_repo.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            price: self.price.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityJobConfig<Perms, E>,
    obligation_repo: ObligationRepo<E>,
    credit_facility_repo: CreditFacilityRepo<E>,
    price: Price,
    audit: Perms::Audit,
}

impl<Perms, E> CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn list_obligations_for_credit_facility(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Vec<Obligation>, CoreCreditError> {
        let mut obligations = vec![];
        let mut query = es_entity::PaginatedQueryArgs::<ObligationsByCreatedAtCursor>::default();
        loop {
            let res = self
                .obligation_repo
                .list_for_credit_facility_id_by_created_at(
                    credit_facility_id,
                    query,
                    es_entity::ListDirection::Ascending,
                )
                .await?;

            obligations.extend(res.entities);

            if res.has_next_page {
                query = es_entity::PaginatedQueryArgs::<ObligationsByCreatedAtCursor> {
                    first: 100,
                    after: res.end_cursor,
                }
            } else {
                break;
            };
        }

        Ok(obligations)
    }
}

#[async_trait]
impl<Perms, E> JobRunner for CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = self.price.usd_cents_per_btc().await?;
        let mut has_next_page = true;
        let mut after: Option<CreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut credit_facilities =
                self.credit_facility_repo
                    .list_by_collateralization_ratio(
                        es_entity::PaginatedQueryArgs::<
                            CreditFacilitiesByCollateralizationRatioCursor,
                        > {
                            first: 10,
                            after,
                        },
                        es_entity::ListDirection::Ascending,
                    )
                    .await?;
            (after, has_next_page) = (
                credit_facilities.end_cursor,
                credit_facilities.has_next_page,
            );
            let mut db = self.credit_facility_repo.begin_op().await?;
            let audit_info = self
                .audit
                .record_system_entry_in_tx(
                    db.tx(),
                    CoreCreditObject::all_credit_facilities(),
                    CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
                )
                .await?;

            let mut at_least_one = false;

            for facility in credit_facilities.entities.iter_mut() {
                if facility.status() == CreditFacilityStatus::Closed {
                    continue;
                }
                let obligations = self
                    .list_obligations_for_credit_facility(facility.id)
                    .await?;
                if facility
                    .maybe_update_collateralization(
                        price,
                        self.config.upgrade_buffer_cvl_pct,
                        &ObligationAggregator::new(
                            obligations
                                .iter()
                                .map(ObligationDataForAggregation::from)
                                .collect::<Vec<_>>(),
                        ),
                        &audit_info,
                    )
                    .is_some()
                {
                    self.credit_facility_repo
                        .update_in_op(&mut db, facility)
                        .await?;
                    at_least_one = true;
                }
            }

            if at_least_one {
                db.commit().await?;
            } else {
                break;
            }
        }

        let now = crate::time::now();
        Ok(JobCompletion::RescheduleAt(now + self.config.job_interval))
    }
}
