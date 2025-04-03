use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;

use crate::{ledger::CreditLedger, obligation::ObligationRepo, primitives::*};

use super::obligation_overdue;

#[derive(Clone, Serialize, Deserialize)]
pub struct CreditFacilityJobConfig<Perms> {
    pub obligation_id: ObligationId,
    pub _phantom: std::marker::PhantomData<Perms>,
}
impl<Perms> JobConfig for CreditFacilityJobConfig<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    type Initializer = CreditFacilityProcessingJobInitializer<Perms>;
}
pub struct CreditFacilityProcessingJobInitializer<Perms>
where
    Perms: PermissionCheck,
{
    obligation_repo: ObligationRepo,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
}

impl<Perms> CreditFacilityProcessingJobInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub fn new(
        ledger: &CreditLedger,
        obligation_repo: ObligationRepo,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligation_repo,
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_DUE_PROCESSING_JOB: JobType = JobType::new("credit-facility-due-processing");
impl<Perms> JobInitializer for CreditFacilityProcessingJobInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_DUE_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms> {
            config: job.config()?,
            obligation_repo: self.obligation_repo.clone(),
            _ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct CreditFacilityProcessingJobRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: CreditFacilityJobConfig<Perms>,
    obligation_repo: ObligationRepo,
    _ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
}

#[async_trait]
impl<Perms> JobRunner for CreditFacilityProcessingJobRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut obligation = self
            .obligation_repo
            .find_by_id(self.config.obligation_id)
            .await?;

        let mut db = self.obligation_repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_obligations(),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await?;

        let _due = if let es_entity::Idempotent::Executed(due) = obligation.record_due(audit_info) {
            due
        } else {
            return Ok(JobCompletion::Complete);
        };

        self.obligation_repo
            .update_in_op(&mut db, &mut obligation)
            .await?;

        self.jobs
            .create_and_spawn_at_in_op(
                &mut db,
                obligation.id,
                obligation_overdue::CreditFacilityJobConfig::<Perms> {
                    obligation_id: obligation.id,
                    _phantom: std::marker::PhantomData,
                },
                obligation.overdue_at(),
            )
            .await?;

        // TODO: switch to recording in ledger and committing
        // self.ledger
        //     .record_due_obligation(db, due)
        //     .await?;
        db.commit().await?;

        Ok(JobCompletion::Complete)
    }
}
