use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;

use crate::{ledger::CreditLedger, obligation::ObligationRepo, primitives::*};

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
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligation_repo,
            audit: audit.clone(),
        }
    }
}

const CREDIT_FACILITY_OVERDUE_PROCESSING_JOB: JobType =
    JobType::new("credit-facility-overdue-processing");
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
        CREDIT_FACILITY_OVERDUE_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms> {
            config: job.config()?,
            obligation_repo: self.obligation_repo.clone(),
            ledger: self.ledger.clone(),
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
    ledger: CreditLedger,
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

        let overdue = if let es_entity::Idempotent::Executed(overdue) =
            obligation.record_overdue_debited_balance(audit_info)?
        {
            overdue
        } else {
            return Ok(JobCompletion::Complete);
        };

        self.obligation_repo
            .update_in_op(&mut db, &mut obligation)
            .await?;

        self.ledger.record_obligation_overdue(db, overdue).await?;

        Ok(JobCompletion::Complete)
    }
}
