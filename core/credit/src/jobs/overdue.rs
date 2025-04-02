use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    credit_facility::CreditFacilityRepo,
    event::CoreCreditEvent,
    ledger::CreditLedger,
    obligation::{Obligation, ObligationRepo},
    primitives::*,
};

use super::obligation_overdue;

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
    obligation_repo: ObligationRepo,
    credit_facility_repo: CreditFacilityRepo<E>,
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
        obligation_repo: ObligationRepo,
        credit_facility_repo: CreditFacilityRepo<E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligation_repo,
            credit_facility_repo,
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

const DISBURSAL_OBLIGATION_OVERDUE_PROCESSING_JOB: JobType =
    JobType::new("disbursal-obligation-overdue-processing");
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
        DISBURSAL_OBLIGATION_OVERDUE_PROCESSING_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityProcessingJobRunner::<Perms, E> {
            config: job.config()?,
            obligation_repo: self.obligation_repo.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
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
    obligation_repo: ObligationRepo,
    credit_facility_repo: CreditFacilityRepo<E>,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
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
        let mut db = self.credit_facility_repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_OVERDUE_DISBURSED_BALANCE,
            )
            .await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(self.config.credit_facility_id)
            .await?;
        let overdue = if let es_entity::Idempotent::Executed((overdue, obligation_ids)) =
            credit_facility.record_overdue_disbursed_balance(audit_info)
        {
            let mut obligations = self
                .obligation_repo
                .find_all::<Obligation>(&obligation_ids)
                .await?;

            for obligation_id in obligation_ids {
                let obligation = obligations
                    .remove(&obligation_id)
                    .expect("Obligation for obligation_id not found in hashmap");

                let overdue_at = obligation
                    .overdue_at()
                    .expect("No overdue_at value set on Obligation");
                self.jobs
                    .create_and_spawn_at_in_op(
                        &mut db,
                        obligation.id,
                        obligation_overdue::CreditFacilityJobConfig::<Perms> {
                            obligation_id: obligation.id,
                            _phantom: std::marker::PhantomData,
                        },
                        overdue_at,
                    )
                    .await?;
            }

            overdue
        } else {
            return Ok(JobCompletion::Complete);
        };

        self.credit_facility_repo
            .update_in_op(&mut db, &mut credit_facility)
            .await?;
        self.ledger
            .record_credit_facility_overdue_disbursed(db, overdue)
            .await?;

        Ok(JobCompletion::Complete)
    }
}
