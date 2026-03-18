//! Complete Accrual Cycle Command Job
//!
//! A single-step command job that finalizes the current interest accrual cycle
//! for a credit facility. This includes:
//! - Recording the interest accrual cycle completion
//! - Creating an interest obligation (if accrued amount > 0)
//! - Starting the next cycle if the facility hasn't matured
//! - Recording the cycle completion to the ledger

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use std::sync::Arc;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use obix::out::OutboxEventMarker;

use core_credit_collateral::{
    CoreCreditCollateralAction, CoreCreditCollateralEvent, CoreCreditCollateralObject,
};
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_eod::complete_accrual_cycle_command::COMPLETE_ACCRUAL_CYCLE_COMMAND_JOB_TYPE;
use core_price::CorePriceEvent;

use core_credit_collection::CoreCreditCollection;

use crate::{
    CompletedAccrualCycle, CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject, CreditFacilityId,
    credit_facility::{
        CreditFacilityRepo, error::CreditFacilityError,
        interest_accrual_cycle::NewInterestAccrualCycleData,
    },
    ledger::*,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteAccrualCycleCommandConfig {
    pub credit_facility_id: CreditFacilityId,
}

pub struct CompleteAccrualCycleCommandInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    ledger: Arc<CreditLedger>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    authz: Arc<Perms>,
}

impl<Perms, E> CompleteAccrualCycleCommandInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        ledger: Arc<CreditLedger>,
        collections: Arc<CoreCreditCollection<Perms, E>>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            ledger,
            collections,
            credit_facility_repo,
            authz,
        }
    }
}

impl<Perms, E> JobInitializer for CompleteAccrualCycleCommandInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = CompleteAccrualCycleCommandConfig;

    fn job_type(&self) -> JobType {
        COMPLETE_ACCRUAL_CYCLE_COMMAND_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CompleteAccrualCycleCommandRunner {
            config: job.config()?,
            collections: self.collections.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
        }))
    }
}

struct CompleteAccrualCycleCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: CompleteAccrualCycleCommandConfig,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

#[async_trait]
impl<Perms, E> JobRunner for CompleteAccrualCycleCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(
        name = "complete_accrual_cycle_command.run",
        skip(self, _current_job),
        fields(credit_facility_id = %self.config.credit_facility_id)
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = self.credit_facility_repo.begin_op().await?;

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
                crate::primitives::INTEREST_ACCRUAL,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let CompletedAccrualCycle {
            facility_accrual_cycle_data,
            new_cycle_data,
        } = self
            .complete_interest_cycle_and_maybe_start_new_cycle_in_op(
                &mut op,
                self.config.credit_facility_id,
            )
            .await?;

        self.ledger
            .record_interest_accrual_cycle_in_op(
                &mut op,
                facility_accrual_cycle_data,
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                    crate::primitives::INTEREST_ACCRUAL,
                ),
            )
            .await?;

        if let Some(NewInterestAccrualCycleData {
            id: new_accrual_cycle_id,
            first_accrual_end_date,
        }) = new_cycle_data
        {
            tracing::info!(
                new_cycle_id = %new_accrual_cycle_id,
                first_accrual_end = %first_accrual_end_date,
                "Cycle completed, next EOD will pick up new cycle"
            );
        } else {
            tracing::info!(
                credit_facility_id = %self.config.credit_facility_id,
                "All interest accrual cycles completed for {}",
                self.config.credit_facility_id
            );
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}

impl<Perms, E> CompleteAccrualCycleCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "credit.facility.complete_interest_cycle_and_maybe_start_new_cycle_in_op",
        skip(self, db)
        fields(credit_facility_id = %credit_facility_id),
    )]
    async fn complete_interest_cycle_and_maybe_start_new_cycle_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
    ) -> Result<CompletedAccrualCycle, CreditFacilityError> {
        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_op(db, credit_facility_id)
            .await?;

        let (accrual_cycle_data, new_obligation) = credit_facility
            .record_interest_accrual_cycle()?
            .expect("record_interest_accrual_cycle should execute when there is an accrual cycle to record");

        if let Some(new_obligation) = new_obligation {
            self.collections
                .obligations()
                .create_in_op(db, new_obligation)
                .await?;
        };

        let res = credit_facility
            .start_interest_accrual_cycle()?
            .expect("start_interest_accrual_cycle always returns Executed");
        self.credit_facility_repo
            .update_in_op(db, &mut credit_facility)
            .await?;

        let new_cycle_data = res.map(|periods| {
            let new_accrual_cycle_id = credit_facility
                .interest_accrual_cycle_in_progress()
                .expect("in-progress accrual cycle must exist after start")
                .id;

            NewInterestAccrualCycleData {
                id: new_accrual_cycle_id,
                first_accrual_end_date: periods.accrual.end,
            }
        });

        Ok(CompletedAccrualCycle {
            facility_accrual_cycle_data: (accrual_cycle_data, credit_facility.account_ids).into(),
            new_cycle_data,
        })
    }
}

pub type CompleteAccrualCycleCommandSpawner = JobSpawner<CompleteAccrualCycleCommandConfig>;
