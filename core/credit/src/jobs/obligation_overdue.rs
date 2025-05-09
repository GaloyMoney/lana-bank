use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    error::CoreCreditError,
    event::CoreCreditEvent,
    ledger::CreditLedger,
    obligation::{Obligation, ObligationOverdueReallocationData, Obligations},
    primitives::*,
    AuditInfo,
};

use super::obligation_defaulted;

#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig<Perms, E> {
    pub obligation_id: ObligationId,
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
    obligations: Obligations<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
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
        obligations: &Obligations<Perms, E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligations: obligations.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_OVERDUE_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-overdue-processing");
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
        CREDIT_FACILITY_OVERDUE_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms, E> {
            config: job.config()?,
            obligations: self.obligations.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
        }))
    }
}

struct RecordedOverdue {
    overdue: ObligationOverdueReallocationData,
    obligation: Obligation,
}

pub struct CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CreditFacilityJobConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
}

impl<Perms, E> CreditFacilityProcessingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[es_entity::retry_on_concurrent_modification]
    async fn record_overdue(
        &self,
        db: &mut es_entity::DbOp<'_>,
        audit_info: &AuditInfo,
    ) -> Result<Option<RecordedOverdue>, CoreCreditError> {
        let mut obligation = self
            .obligations
            .find_by_id(self.config.obligation_id)
            .await?;

        if let es_entity::Idempotent::Executed(overdue) =
            obligation.record_overdue(audit_info.clone())?
        {
            self.obligations.update_in_op(db, &mut obligation).await?;

            Ok(Some(RecordedOverdue {
                overdue,
                obligation,
            }))
        } else {
            Ok(None)
        }
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
        let mut db = self.obligations.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_obligations(),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await?;

        if let Some(recorded) = self.record_overdue(&mut db, &audit_info).await? {
            if let Some(defaulted_at) = recorded.obligation.defaulted_at() {
                self.jobs
                    .create_and_spawn_at_in_op(
                        &mut db,
                        JobId::new(),
                        obligation_defaulted::CreditFacilityJobConfig::<Perms, E> {
                            obligation_id: recorded.obligation.id,
                            _phantom: std::marker::PhantomData,
                        },
                        defaulted_at,
                    )
                    .await?;
            }

            self.ledger
                .record_obligation_overdue(db, recorded.overdue)
                .await?;
        }

        Ok(JobCompletion::Complete)
    }
}
