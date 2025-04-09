use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo,
    error::CoreCreditError,
    event::CoreCreditEvent,
    ledger::*,
    obligation::{obligation_cursor::ObligationsByCreatedAtCursor, Obligation, ObligationRepo},
    obligation_aggregator::{ObligationAggregator, ObligationDataForAggregation},
    primitives::*,
    terms::InterestPeriod,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
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
    ledger: CreditLedger,
    obligation_repo: ObligationRepo,
    credit_facility_repo: CreditFacilityRepo<E>,
    audit: Perms::Audit,
    jobs: Jobs,
}

impl<Perms, E> CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        ledger: &CreditLedger,
        obligation_repo: ObligationRepo,
        credit_facility_repo: CreditFacilityRepo<E>,
        audit: &Perms::Audit,
        jobs: &Jobs,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligation_repo,
            credit_facility_repo,
            audit: audit.clone(),
            jobs: jobs.clone(),
        }
    }
}

const CREDIT_FACILITY_INTEREST_ACCRUAL_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-interest-accrual-processing");
impl<Perms, E> JobInitializer for CreditFacilityProcessingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_INTEREST_ACCRUAL_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms, E> {
            config: job.config()?,
            obligation_repo: self.obligation_repo.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            audit: self.audit.clone(),
            jobs: self.jobs.clone(),
        }))
    }
}

#[derive(Clone)]
struct ConfirmedAccrual {
    accrual: CreditFacilityInterestAccrual,
    next_period: Option<InterestPeriod>,
    accrual_idx: InterestAccrualCycleIdx,
}

pub struct CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityJobConfig<Perms, E>,
    obligation_repo: ObligationRepo,
    credit_facility_repo: CreditFacilityRepo<E>,
    ledger: CreditLedger,
    audit: Perms::Audit,
    jobs: Jobs,
}

impl<Perms, E> CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[es_entity::retry_on_concurrent_modification]
    async fn confirm_interest_accrual(
        &self,
        db: &mut es_entity::DbOp<'_>,
        audit_info: &AuditInfo,
    ) -> Result<ConfirmedAccrual, CoreCreditError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;

        let obligations = self
            .list_obligations_for_credit_facility(self.config.credit_facility_id)
            .await?;

        let confirmed_accrual = {
            let outstanding = ObligationAggregator::new(
                obligations
                    .iter()
                    .map(ObligationDataForAggregation::from)
                    .collect::<Vec<_>>(),
            )
            .outstanding();

            let account_ids = credit_facility.account_ids;

            let accrual = credit_facility
                .interest_accrual_cycle_in_progress_mut()
                .expect("Accrual in progress should exist for scheduled job");

            let interest_accrual = accrual.record_accrual(outstanding.overdue, audit_info.clone());

            ConfirmedAccrual {
                accrual: (interest_accrual, account_ids).into(),
                next_period: accrual.next_accrual_period(),
                accrual_idx: accrual.idx,
            }
        };

        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        Ok(confirmed_accrual)
    }

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
    #[instrument(
        name = "credit-facility.interest-accruals.job",
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
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let ConfirmedAccrual {
            accrual: interest_accrual,
            next_period: next_accrual_period,
            accrual_idx,
        } = self.confirm_interest_accrual(&mut db, &audit_info).await?;

        let (now, mut tx) = (db.now(), db.into_tx());
        let sub_op = {
            use sqlx::Acquire;
            es_entity::DbOp::new(tx.begin().await?, now)
        };
        self.ledger
            .record_interest_accrual(sub_op, interest_accrual)
            .await?;

        let mut db = es_entity::DbOp::new(tx, now);
        if let Some(period) = next_accrual_period {
            Ok(JobCompletion::RescheduleAtWithOp(db, period.end))
        } else {
            self.jobs
                .create_and_spawn_in_op(
                    &mut db,
                    uuid::Uuid::new_v4(),
                    super::interest_accrual_cycles::CreditFacilityJobConfig::<Perms, E> {
                        credit_facility_id: self.config.credit_facility_id,
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
            println!(
            "Credit Facility interest accruals job completed for accrual index {:?} for credit_facility: {:?}",
            accrual_idx, self.config.credit_facility_id
        );
            Ok(JobCompletion::CompleteWithOp(db))
        }
    }
}
