use std::{collections::HashSet, sync::Arc};

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
    Collecting(CreditFacilityEodCollectingState),
    Tracking {
        pending: HashSet<CreditFacilityId>,
        start_sequence: i64,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreditFacilityEodCollectingState {
    accrual_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    maturity_cursor: Option<(chrono::DateTime<chrono::Utc>, CreditFacilityId)>,
    pending: HashSet<CreditFacilityId>,
    accrual_done: bool,
}

impl<E> CreditFacilityEodProcessRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    async fn run_collecting(
        &self,
        mut current_job: CurrentJob,
        mut state: CreditFacilityEodCollectingState,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Phase: Collect accrual facilities
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
                            &CreditFacilityEodState::Collecting(state.clone()),
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
                        state.pending.insert(*id);
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
                        &CreditFacilityEodState::Collecting(state.clone()),
                    )
                    .await?;
                op.commit().await?;
            }
        }

        // Phase: Collect maturing facilities
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
                    state.pending.insert(*id);
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
                    &CreditFacilityEodState::Collecting(state.clone()),
                )
                .await?;
            op.commit().await?;
        }

        let start_sequence = self.outbox.current_sequence().await?;

        tracing::info!(
            entities = state.pending.len(),
            start_sequence,
            "Credit facility EOD collection complete, transitioning to tracking"
        );

        // Transition to Tracking state
        let new_state = CreditFacilityEodState::Tracking {
            pending: state.pending,
            start_sequence,
        };
        let mut op = current_job.begin_op().await?;
        current_job
            .update_execution_state_in_op(&mut op, &new_state)
            .await?;
        Ok(JobCompletion::RescheduleNowWithOp(op))
    }

    fn extract_facility_completion(event: &CoreCreditEvent) -> Option<CreditFacilityId> {
        match event {
            CoreCreditEvent::AccrualPosted { entity } => Some(entity.credit_facility_id),
            CoreCreditEvent::FacilityMatured { entity } => Some(entity.id),
            _ => None,
        }
    }

    async fn run_tracking(
        &self,
        mut current_job: CurrentJob,
        mut pending: HashSet<CreditFacilityId>,
        mut start_sequence: i64,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if pending.is_empty() {
            tracing::info!("No credit facilities to track, completing immediately");
            return Ok(JobCompletion::Complete);
        }

        tracing::info!(
            remaining = pending.len(),
            start_sequence,
            "Streaming outbox events for credit facility EOD completion"
        );

        let mut stream = self.outbox.listen_persisted(Some(start_sequence));

        loop {
            tokio::select! {
                Some(event) = stream.next() => {
                    if let Some(payload) = event.payload.as_ref() {
                        if let Some(credit_event) = payload.as_event::<CoreCreditEvent>() {
                            if let Some(facility_id) = Self::extract_facility_completion(credit_event) {
                                if pending.remove(&facility_id) {
                                    start_sequence = event.sequence;
                                    let state = CreditFacilityEodState::Tracking {
                                        pending: pending.clone(),
                                        start_sequence,
                                    };
                                    current_job.update_execution_state(&state).await?;
                                }
                            }
                        }
                    }
                    if pending.is_empty() {
                        tracing::info!("All credit facility EOD per-entity jobs completed");
                        return Ok(JobCompletion::Complete);
                    }
                }
                _ = current_job.shutdown_requested() => {
                    let state = CreditFacilityEodState::Tracking {
                        pending,
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
            CreditFacilityEodState::Collecting(collecting) => {
                self.run_collecting(current_job, collecting).await
            }
            CreditFacilityEodState::Tracking {
                pending,
                start_sequence,
            } => {
                self.run_tracking(current_job, pending, start_sequence)
                    .await
            }
        }
    }
}
