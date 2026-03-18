use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use governance::GovernanceEvent;
use job::{error::JobError, *};
use obix::out::{Outbox, OutboxEventMarker};

use core_custody::CoreCustodyEvent;
use core_eod::credit_facility_eod_process::{
    CREDIT_FACILITY_EOD_PROCESS_JOB_TYPE, CreditFacilityEodProcessConfig,
};
use core_price::CorePriceEvent;

use super::credit_facility_maturity::{
    CreditFacilityMaturityJobConfig, CreditFacilityMaturityJobSpawner,
};
use super::process_accrual_cycle::{ProcessAccrualCycleJobConfig, ProcessAccrualCycleJobSpawner};
use crate::{CoreCreditEvent, CreditFacilityId, credit_facility::CreditFacilityRepo};

const PAGE_SIZE: i64 = 100;

pub struct CreditFacilityEodProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
    maturity_spawner: CreditFacilityMaturityJobSpawner,
}

impl<E> CreditFacilityEodProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
        maturity_spawner: CreditFacilityMaturityJobSpawner,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            credit_facility_repo,
            process_accrual_cycle_spawner,
            maturity_spawner,
        }
    }
}

impl<E> JobInitializer for CreditFacilityEodProcessInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = CreditFacilityEodProcessConfig;

    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_EOD_PROCESS_JOB_TYPE
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityEodProcessRunner {
            config: job.config()?,
            outbox: self.outbox.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            process_accrual_cycle_spawner: self.process_accrual_cycle_spawner.clone(),
            maturity_spawner: self.maturity_spawner.clone(),
        }))
    }
}

struct CreditFacilityEodProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: CreditFacilityEodProcessConfig,
    outbox: Outbox<E>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    process_accrual_cycle_spawner: ProcessAccrualCycleJobSpawner,
    maturity_spawner: CreditFacilityMaturityJobSpawner,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CreditFacilityEodState {
    #[default]
    SpawningAccrualAndMaturityJobs(SpawningAccrualAndMaturityJobsState),
    AwaitingAccrualsAndMaturities {
        pending_accrual: HashSet<CreditFacilityId>,
        pending_maturity: HashSet<CreditFacilityId>,
        start_sequence: i64,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpawningAccrualAndMaturityJobsState {
    accrual_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    maturity_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    pending_accrual: HashSet<CreditFacilityId>,
    pending_maturity: HashSet<CreditFacilityId>,
    accrual_done: bool,
}

impl<E> CreditFacilityEodProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    /// Step 1: Capture the outbox sequence, then query facilities eligible for
    /// accrual and maturity (paginated) and spawn per-facility jobs for each.
    /// Transitions to AwaitingAccrualsAndMaturities when all pages are processed.
    async fn spawn_accrual_and_maturity_jobs(
        &self,
        mut current_job: CurrentJob,
        mut state: SpawningAccrualAndMaturityJobsState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Capture sequence BEFORE spawning any jobs so we never miss a
        // completion event from a fast-finishing job.
        let start_sequence = self.outbox.current_sequence().await?;

        // Phase: Spawn accrual jobs for eligible facilities
        if !state.accrual_done {
            loop {
                let mut op = current_job.begin_op().await?;

                let rows = self
                    .credit_facility_repo
                    .list_facility_ids_eligible_for_accrual_in_op(
                        &mut op,
                        self.config.date,
                        state.accrual_cursor,
                        PAGE_SIZE,
                    )
                    .await?;

                if rows.is_empty() {
                    state.accrual_done = true;
                    current_job
                        .update_execution_state_in_op(
                            &mut op,
                            &CreditFacilityEodState::SpawningAccrualAndMaturityJobs(state.clone()),
                        )
                        .await?;
                    op.commit().await?;
                    break;
                }

                let specs: Vec<_> = rows
                    .iter()
                    .map(|(id, _)| {
                        let job_id = core_eod::eod_entity_id(
                            &self.config.date,
                            "interest-accrual",
                            &(*id).into(),
                        );
                        state.pending_accrual.insert(*id);
                        JobSpec::new(
                            job_id,
                            ProcessAccrualCycleJobConfig {
                                credit_facility_id: *id,
                            },
                        )
                        .queue_id(id.to_string())
                    })
                    .collect();

                match self
                    .process_accrual_cycle_spawner
                    .spawn_all_in_op(&mut op, specs)
                    .await
                {
                    Ok(_) | Err(JobError::DuplicateId(_)) => {}
                    Err(e) => return Err(e.into()),
                }

                state.accrual_cursor = rows.last().map(|(id, ts)| (*ts, *id));
                current_job
                    .update_execution_state_in_op(
                        &mut op,
                        &CreditFacilityEodState::SpawningAccrualAndMaturityJobs(state.clone()),
                    )
                    .await?;
                op.commit().await?;
            }
        }

        // Phase: Spawn maturity jobs for facilities ready to mature
        loop {
            let mut op = current_job.begin_op().await?;

            let rows = self
                .credit_facility_repo
                .list_ids_ready_for_maturity_in_op(
                    &mut op,
                    self.config.date,
                    state.maturity_cursor,
                    PAGE_SIZE,
                )
                .await?;

            if rows.is_empty() {
                break;
            }

            let specs: Vec<_> = rows
                .iter()
                .map(|(id, _)| {
                    let job_id = core_eod::eod_entity_id(
                        &self.config.date,
                        "credit-maturity",
                        &(*id).into(),
                    );
                    state.pending_maturity.insert(*id);
                    JobSpec::new(
                        job_id,
                        CreditFacilityMaturityJobConfig {
                            credit_facility_id: *id,
                        },
                    )
                    .queue_id(id.to_string())
                })
                .collect();

            match self.maturity_spawner.spawn_all_in_op(&mut op, specs).await {
                Ok(_) | Err(JobError::DuplicateId(_)) => {}
                Err(e) => return Err(e.into()),
            }

            state.maturity_cursor = rows.last().map(|(id, ts)| (*ts, *id));
            current_job
                .update_execution_state_in_op(
                    &mut op,
                    &CreditFacilityEodState::SpawningAccrualAndMaturityJobs(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        let total = state.pending_accrual.len() + state.pending_maturity.len();
        tracing::info!(
            accruals = state.pending_accrual.len(),
            maturities = state.pending_maturity.len(),
            total,
            start_sequence,
            "Credit facility EOD spawning complete, transitioning to awaiting"
        );

        let new_state = CreditFacilityEodState::AwaitingAccrualsAndMaturities {
            pending_accrual: state.pending_accrual,
            pending_maturity: state.pending_maturity,
            start_sequence,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    /// Step 2: Stream outbox events from the saved sequence, matching accrual
    /// and maturity completion events. Removes completed facilities from their
    /// respective pending sets and checkpoints on each match. Completes when
    /// both sets are empty.
    async fn await_accrual_and_maturity_events(
        &self,
        mut current_job: CurrentJob,
        mut pending_accrual: HashSet<CreditFacilityId>,
        mut pending_maturity: HashSet<CreditFacilityId>,
        mut start_sequence: i64,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if pending_accrual.is_empty() && pending_maturity.is_empty() {
            tracing::info!("No credit facilities to track, completing immediately");
            return Ok(JobCompletion::Complete);
        }

        tracing::info!(
            remaining_accruals = pending_accrual.len(),
            remaining_maturities = pending_maturity.len(),
            start_sequence,
            "Streaming outbox events for credit facility EOD completion"
        );

        let mut stream = self.outbox.listen_persisted(Some(start_sequence));

        loop {
            tokio::select! {
                Some(event) = stream.next() => {
                    if let Some(payload) = event.payload.as_ref() {
                        if let Some(credit_event) = payload.as_event::<CoreCreditEvent>() {
                            let changed = match credit_event {
                                CoreCreditEvent::AccrualPosted { entity } => {
                                    pending_accrual.remove(&entity.credit_facility_id)
                                }
                                CoreCreditEvent::FacilityMatured { entity } => {
                                    pending_maturity.remove(&entity.id)
                                }
                                _ => false,
                            };
                            if changed {
                                start_sequence = event.sequence;
                                let state = CreditFacilityEodState::AwaitingAccrualsAndMaturities {
                                    pending_accrual: pending_accrual.clone(),
                                    pending_maturity: pending_maturity.clone(),
                                    start_sequence,
                                };
                                current_job.update_execution_state(&state).await?;
                            }
                        }
                    }
                    if pending_accrual.is_empty() && pending_maturity.is_empty() {
                        tracing::info!("All credit facility EOD per-entity jobs completed");
                        return Ok(JobCompletion::Complete);
                    }
                }
                _ = current_job.shutdown_requested() => {
                    let state = CreditFacilityEodState::AwaitingAccrualsAndMaturities {
                        pending_accrual,
                        pending_maturity,
                        start_sequence,
                    };
                    current_job.update_execution_state(&state).await?;
                    tracing::info!("Shutdown requested, rescheduling credit facility EOD tracking");
                    return Ok(JobCompletion::RescheduleIn(std::time::Duration::ZERO));
                }
            }
        }
    }
}

#[async_trait]
impl<E> JobRunner for CreditFacilityEodProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[instrument(
        name = "eod.credit-facility-eod-process.run",
        skip(self, current_job),
        fields(date = %self.config.date)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<CreditFacilityEodState>()?
            .unwrap_or_default();

        match state {
            CreditFacilityEodState::SpawningAccrualAndMaturityJobs(spawning) => {
                self.spawn_accrual_and_maturity_jobs(current_job, spawning)
                    .await
            }
            CreditFacilityEodState::AwaitingAccrualsAndMaturities {
                pending_accrual,
                pending_maturity,
                start_sequence,
            } => {
                self.await_accrual_and_maturity_events(
                    current_job,
                    pending_accrual,
                    pending_maturity,
                    start_sequence,
                )
                .await
            }
        }
    }
}
