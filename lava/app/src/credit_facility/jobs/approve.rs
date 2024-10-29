use async_trait::async_trait;
use futures::StreamExt;

use governance::GovernanceEvent;
use job::*;
use lava_events::LavaEvent;
use rbac_types::{AppObject, CreditFacilityAction};

use crate::{
    audit::{Audit, AuditSvc},
    credit_facility::{repo::CreditFacilityRepo, APPROVE_CREDIT_FACILITY_PROCESS},
    ledger::Ledger,
    outbox::Outbox,
    price::Price,
};

#[derive(serde::Serialize)]
pub(crate) struct CreditFacilityApprovalJobConfig;
impl JobConfig for CreditFacilityApprovalJobConfig {
    type Initializer = CreditFacilityApprovalJobInitializer;
}

pub(crate) struct CreditFacilityApprovalJobInitializer {
    pool: sqlx::PgPool,
    repo: CreditFacilityRepo,
    price: Price,
    ledger: Ledger,
    audit: Audit,
    outbox: Outbox,
}

impl CreditFacilityApprovalJobInitializer {
    pub fn new(
        pool: &sqlx::PgPool,
        repo: &CreditFacilityRepo,
        price: &Price,
        ledger: &Ledger,
        audit: &Audit,
        outbox: &Outbox,
    ) -> Self {
        Self {
            pool: pool.clone(),
            repo: repo.clone(),
            price: price.clone(),
            ledger: ledger.clone(),
            audit: audit.clone(),
            outbox: outbox.clone(),
        }
    }
}

const CREDIT_FACILITY_APPROVE_JOB: JobType = JobType::new("credit-facility-approve");
impl JobInitializer for CreditFacilityApprovalJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityApprovalJobRunner {
            pool: self.pool.clone(),
            repo: self.repo.clone(),
            price: self.price.clone(),
            ledger: self.ledger.clone(),
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
struct CreditFacilityApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct CreditFacilityApprovalJobRunner {
    pool: sqlx::PgPool,
    repo: CreditFacilityRepo,
    price: Price,
    ledger: Ledger,
    audit: Audit,
    outbox: Outbox,
}
#[async_trait]
impl JobRunner for CreditFacilityApprovalJobRunner {
    #[allow(clippy::single_match)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.payload {
                Some(LavaEvent::Governance(GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    ref process_type,
                })) if process_type == &APPROVE_CREDIT_FACILITY_PROCESS => {
                    let mut db_tx = self.pool.begin().await?;

                    let mut credit_facility = self.repo.find_by_approval_process_id(id).await?;
                    let audit_info = self
                        .audit
                        .record_system_entry_in_tx(
                            &mut db_tx,
                            AppObject::CreditFacility,
                            CreditFacilityAction::ConcludeApprovalProcess,
                        )
                        .await?;
                    credit_facility.approval_process_concluded(approved, audit_info);

                    let price = self.price.usd_cents_per_btc().await?;
                    if let Ok(credit_facility_activation) = credit_facility.activation_data(price) {
                        self.ledger
                            .activate_credit_facility(credit_facility_activation.clone())
                            .await?;
                        let audit_info = self
                            .audit
                            .record_system_entry_in_tx(
                                &mut db_tx,
                                AppObject::CreditFacility,
                                CreditFacilityAction::Activate,
                            )
                            .await?;
                        credit_facility.activate(
                            credit_facility_activation,
                            chrono::Utc::now(),
                            audit_info,
                        );
                    }

                    self.repo
                        .update_in_tx(&mut db_tx, &mut credit_facility)
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
