//! Accrue Interest Command Job
//!
//! A single-step command job that calculates and records interest for one
//! accrual period within a credit facility's interest accrual cycle.
//!
//! This job handles a single period: if more periods remain, the parent
//! InterestAccrualProcess sees the completion and the CreditFacilityEodProcess
//! will spawn a new InterestAccrualProcess on the next EOD.

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
    Collaterals, CoreCreditCollateralAction, CoreCreditCollateralEvent, CoreCreditCollateralObject,
};
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_eod::accrue_interest_command::ACCRUE_INTEREST_COMMAND_JOB_TYPE;
use core_price::CorePriceEvent;

use crate::{
    AccrualOutcome, CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId,
    credit_facility::{
        CreditFacilityRepo, error::CreditFacilityError,
        interest_accrual_cycle::error::InterestAccrualCycleError,
    },
    ledger::*,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccrueInterestCommandConfig {
    pub credit_facility_id: CreditFacilityId,
}

pub struct AccrueInterestCommandInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    ledger: Arc<CreditLedger>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    authz: Arc<Perms>,
}

impl<Perms, E> AccrueInterestCommandInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        ledger: Arc<CreditLedger>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            ledger,
            credit_facility_repo,
            collaterals,
            authz,
        }
    }
}

impl<Perms, E> JobInitializer for AccrueInterestCommandInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = AccrueInterestCommandConfig;

    fn job_type(&self) -> JobType {
        ACCRUE_INTEREST_COMMAND_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(AccrueInterestCommandRunner {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            collaterals: self.collaterals.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
        }))
    }
}

struct AccrueInterestCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: AccrueInterestCommandConfig,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

#[async_trait]
impl<Perms, E> JobRunner for AccrueInterestCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "accrue_interest_command.run",
        skip(self, _current_job),
        fields(credit_facility_id = %self.config.credit_facility_id)
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut db = self.credit_facility_repo.begin_op().await?;

        let outcome = self
            .confirm_interest_accrual_in_op(&mut db, self.config.credit_facility_id)
            .await?;

        match outcome {
            AccrualOutcome::Accrued(confirmed) => {
                self.ledger
                    .record_interest_accrual_in_op(
                        &mut db,
                        confirmed.accrual,
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                            crate::primitives::INTEREST_ACCRUAL,
                        ),
                    )
                    .await?;

                if confirmed.next_period.is_some() {
                    tracing::debug!(
                        accrual_idx = %confirmed.accrual_idx,
                        accrued_count = %confirmed.accrued_count,
                        "Period accrued, more periods remain in cycle"
                    );
                } else {
                    tracing::info!(
                        accrued_count = %confirmed.accrued_count,
                        accrual_idx = %confirmed.accrual_idx,
                        "All periods accrued in cycle"
                    );
                }

                Ok(JobCompletion::CompleteWithOp(db))
            }
            AccrualOutcome::AllPeriodsComplete => {
                tracing::info!(
                    credit_facility_id = %self.config.credit_facility_id,
                    "All periods already accrued"
                );
                Ok(JobCompletion::CompleteWithOp(db))
            }
            AccrualOutcome::NoCycleInProgress => {
                tracing::info!(
                    credit_facility_id = %self.config.credit_facility_id,
                    "No accrual cycle in progress, completing"
                );
                Ok(JobCompletion::CompleteWithOp(db))
            }
        }
    }
}

impl<Perms, E> AccrueInterestCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.confirm_interest_accrual_in_op",
        skip(self, op),
        fields(credit_facility_id = %credit_facility_id)
    )]
    async fn confirm_interest_accrual_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        credit_facility_id: CreditFacilityId,
    ) -> Result<AccrualOutcome, CreditFacilityError> {
        self.authz
            .audit()
            .record_system_entry_in_op(
                op,
                crate::primitives::INTEREST_ACCRUAL,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_op(op, credit_facility_id)
            .await?;

        let account_ids = credit_facility.account_ids;
        let collateral_account_id = self
            .collaterals
            .collateral_ledger_account_ids_in_op(op, credit_facility.collateral_id)
            .await?
            .collateral_account_id;

        let balances = self
            .ledger
            .get_credit_facility_balance_in_op(op, account_ids, collateral_account_id)
            .await?;

        let result = match credit_facility
            .record_accrual_on_in_progress_cycle(balances.disbursed_outstanding())
        {
            Ok(recorded) => {
                let recorded = recorded.expect("record_accrual always returns Executed");
                AccrualOutcome::Accrued(crate::ConfirmedAccrual {
                    accrual: (recorded.accrual_data, account_ids).into(),
                    next_period: recorded.next_period,
                    accrual_idx: recorded.accrual_idx,
                    accrued_count: recorded.accrued_count,
                })
            }
            Err(CreditFacilityError::NoAccrualCycleInProgress) => AccrualOutcome::NoCycleInProgress,
            Err(CreditFacilityError::InterestAccrualCycleError(
                InterestAccrualCycleError::NoNextAccrualPeriod,
            )) => AccrualOutcome::AllPeriodsComplete,
            Err(e) => return Err(e),
        };

        self.credit_facility_repo
            .update_in_op(op, &mut credit_facility)
            .await?;

        Ok(result)
    }
}

pub type AccrueInterestCommandSpawner = JobSpawner<AccrueInterestCommandConfig>;
