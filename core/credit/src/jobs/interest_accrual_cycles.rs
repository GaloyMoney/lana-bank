use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId,
    credit_facility::CreditFacilities, interest_accruals, ledger::*, obligation::Obligations,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct InterestAccrualCycleJobConfig<Perms, E> {
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for InterestAccrualCycleJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = InterestAccrualCycleJobInitializer<Perms, E>;
}

pub struct InterestAccrualCycleJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    ledger: CreditLedger,
    obligations: Obligations<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    jobs: Jobs,
    audit: Perms::Audit,
}

impl<Perms, E> InterestAccrualCycleJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        ledger: &CreditLedger,
        obligations: &Obligations<Perms, E>,
        credit_facilities: &CreditFacilities<Perms, E>,
        jobs: &Jobs,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            ledger: ledger.clone(),
            obligations: obligations.clone(),
            credit_facilities: credit_facilities.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

const INTEREST_ACCRUAL_CYCLE_JOB: JobType = JobType::new("interest-accrual-cycle");
impl<Perms, E> JobInitializer for InterestAccrualCycleJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        INTEREST_ACCRUAL_CYCLE_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(InterestAccrualCycleJobRunner::<Perms, E> {
            config: job.config()?,
            obligations: self.obligations.clone(),
            credit_facilities: self.credit_facilities.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct InterestAccrualCycleJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    config: InterestAccrualCycleJobConfig<Perms, E>,
    obligations: Obligations<Perms, E>,
    credit_facilities: CreditFacilities<Perms, E>,
    ledger: CreditLedger,
    jobs: Jobs,
    audit: Perms::Audit,
}

#[async_trait]
impl<Perms, E> JobRunner for InterestAccrualCycleJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(
        name = "credit-facility.interest-accrual-cycles.job",
        skip(self, current_job),
        fields(attempt)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let span = tracing::Span::current();
        span.record("attempt", current_job.attempt());

        if !self
            .obligations
            .check_facility_obligations_status_updated(self.config.credit_facility_id)
            .await?
        {
            return Ok(JobCompletion::RescheduleIn(
                chrono::Duration::minutes(5).to_std()?,
            ));
        }

        let mut db = self.credit_facilities.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_RECORD_INTEREST,
            )
            .await?;

        let (obligation, new_cycle_data) = self
            .credit_facilities
            .complete_interest_cycle_and_maybe_start_new_cycle(
                &mut db,
                self.config.credit_facility_id,
                &audit_info,
            )
            .await?;

        if let Some((new_accrual_cycle_id, first_accrual_end_date)) = new_cycle_data {
            self.jobs
                .create_and_spawn_at_in_op(
                    &mut db,
                    new_accrual_cycle_id,
                    interest_accruals::InterestAccrualJobConfig::<Perms, E> {
                        credit_facility_id: self.config.credit_facility_id,
                        _phantom: std::marker::PhantomData,
                    },
                    first_accrual_end_date,
                )
                .await?;
        } else {
            println!(
                "All Credit Facility interest accrual cycles completed for credit_facility: {:?}",
                self.config.credit_facility_id
            );
        };

        self.ledger
            .record_interest_accrual_cycle(db, obligation)
            .await?;

        return Ok(JobCompletion::Complete);
    }
}
