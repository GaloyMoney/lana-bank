use async_trait::async_trait;
use futures::StreamExt;

use governance::GovernanceEvent;
use job::*;
use lava_events::LavaEvent;
use rbac_types::{AppObject, CreditFacilityAction};

use crate::{
    audit::{Audit, AuditSvc},
    credit_facility::{DisbursementRepo, APPROVE_DISBURSEMENT_PROCESS},
    outbox::Outbox,
};

#[derive(serde::Serialize)]
pub(crate) struct DisbursementApprovalJobConfig;
impl JobConfig for DisbursementApprovalJobConfig {
    type Initializer = DisbursementApprovalJobInitializer;
}

pub(crate) struct DisbursementApprovalJobInitializer {
    pool: sqlx::PgPool,
    disbursement_repo: DisbursementRepo,
    audit: Audit,
    outbox: Outbox,
}

impl DisbursementApprovalJobInitializer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: &sqlx::PgPool,
        disbursement_repo: &DisbursementRepo,
        audit: &Audit,
        outbox: &Outbox,
    ) -> Self {
        Self {
            pool: pool.clone(),
            disbursement_repo: disbursement_repo.clone(),
            audit: audit.clone(),
            outbox: outbox.clone(),
        }
    }
}

const DISBURSEMENT_APPROVE_JOB: JobType = JobType::new("disbursement-approve");
impl JobInitializer for DisbursementApprovalJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DISBURSEMENT_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DisbursementApprovalJobRunner {
            pool: self.pool.clone(),
            disbursement_repo: self.disbursement_repo.clone(),
            audit: self.audit.clone(),
            outbox: self.outbox.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct DisbursementApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct DisbursementApprovalJobRunner {
    pool: sqlx::PgPool,
    disbursement_repo: DisbursementRepo,
    audit: Audit,
    outbox: Outbox,
}
#[async_trait]
impl JobRunner for DisbursementApprovalJobRunner {
    #[allow(clippy::single_match)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<DisbursementApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.payload {
                Some(LavaEvent::Governance(GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    ref process_type,
                })) if process_type == &APPROVE_DISBURSEMENT_PROCESS => {
                    let mut db_tx = self.pool.begin().await?;

                    let mut disbursement = self
                        .disbursement_repo
                        .find_by_approval_process_id(id)
                        .await?;
                    let audit_info = self
                        .audit
                        .record_system_entry_in_tx(
                            &mut db_tx,
                            AppObject::CreditFacility,
                            CreditFacilityAction::ConcludeDisbursementApprovalProcess,
                        )
                        .await?;
                    disbursement.approval_process_concluded(approved, audit_info);

                    self.disbursement_repo
                        .update_in_tx(&mut db_tx, &mut disbursement)
                        .await?;
                    state.sequence = message.sequence;
                    current_job
                        .update_execution_state(&mut db_tx, state)
                        .await?;
                    db_tx.commit().await?;
                }
                _ => {}
            }
        }

        Ok(JobCompletion::RescheduleAt(chrono::Utc::now()))
    }
}
