use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_time_events::CoreTimeEvent;
use job::*;
use obix::out::OutboxEventMarker;

use super::interest_accrual::{InterestAccrualJobConfig, InterestAccrualJobSpawner};
use crate::{
    CoreCreditEvent, CreditFacilityId, credit_facility::CreditFacilityRepo, primitives::*,
};

const COLLECT_FACILITIES_FOR_ACCRUAL_JOB: JobType =
    JobType::new("task.collect-facilities-for-accrual");
const PAGE_SIZE: i64 = 100;

#[derive(Serialize, Deserialize)]
pub struct CollectFacilitiesForAccrualJobConfig<Perms, E> {
    pub day: chrono::NaiveDate,
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for CollectFacilitiesForAccrualJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            day: self.day,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct CollectFacilitiesForAccrualJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    credit_facility_repo: CreditFacilityRepo<E>,
    interest_accrual_spawner: InterestAccrualJobSpawner<Perms, E>,
}

impl<Perms, E> CollectFacilitiesForAccrualJobInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        credit_facility_repo: &CreditFacilityRepo<E>,
        interest_accrual_spawner: InterestAccrualJobSpawner<Perms, E>,
    ) -> Self {
        Self {
            credit_facility_repo: credit_facility_repo.clone(),
            interest_accrual_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for CollectFacilitiesForAccrualJobInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = CollectFacilitiesForAccrualJobConfig<Perms, E>;

    fn job_type(&self) -> JobType {
        COLLECT_FACILITIES_FOR_ACCRUAL_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CollectFacilitiesForAccrualJobRunner {
            config: job.config()?,
            credit_facility_repo: self.credit_facility_repo.clone(),
            interest_accrual_spawner: self.interest_accrual_spawner.clone(),
        }))
    }
}

struct CollectFacilitiesForAccrualJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: CollectFacilitiesForAccrualJobConfig<Perms, E>,
    credit_facility_repo: CreditFacilityRepo<E>,
    interest_accrual_spawner: InterestAccrualJobSpawner<Perms, E>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct CollectFacilitiesForAccrualState {
    last_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
}

#[async_trait]
impl<Perms, E> JobRunner for CollectFacilitiesForAccrualJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(
        name = "credit.credit_facility.collect_facilities_for_accrual_job",
        skip(self, current_job),
        fields(day = %self.config.day)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CollectFacilitiesForAccrualState>()?
            .unwrap_or_default();

        loop {
            let rows = self
                .credit_facility_repo
                .list_facility_ids_eligible_for_accrual(
                    self.config.day,
                    state.last_cursor,
                    PAGE_SIZE,
                )
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    JobSpec::new(
                        JobId::new(),
                        InterestAccrualJobConfig::<Perms, E> {
                            credit_facility_id: *id,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            let mut op = current_job.begin_op().await?;
            self.interest_accrual_spawner
                .spawn_all_in_op(&mut op, specs)
                .await?;

            state.last_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(&mut op, &state)
                .await?;
            op.commit().await?;
        }

        Ok(JobCompletion::Complete)
    }
}

pub type CollectFacilitiesForAccrualJobSpawner<Perms, E> =
    JobSpawner<CollectFacilitiesForAccrualJobConfig<Perms, E>>;
