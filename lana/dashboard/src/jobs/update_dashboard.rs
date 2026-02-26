use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use job::*;
use lana_events::{CollateralDirection, ObligationType};
use money::{Satoshis, UsdCents};
use tracing_macros::record_error_severity;

use crate::repo::DashboardRepo;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UpdateDashboardUpdate {
    FacilityProposalCreated {
        recorded_at: DateTime<Utc>,
    },
    FacilityActivated {
        recorded_at: DateTime<Utc>,
    },
    FacilityCompleted {
        recorded_at: DateTime<Utc>,
    },
    DisbursalSettled {
        recorded_at: DateTime<Utc>,
        amount: UsdCents,
    },
    PaymentAllocationCreated {
        recorded_at: DateTime<Utc>,
        amount: UsdCents,
        obligation_type: ObligationType,
    },
    CollateralUpdated {
        recorded_at: DateTime<Utc>,
        direction: CollateralDirection,
        abs_diff: Satoshis,
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDashboardConfig {
    pub update: UpdateDashboardUpdate,
    pub trace_context: tracing_utils::persistence::SerializableTraceContext,
}

pub const UPDATE_DASHBOARD_COMMAND: JobType = JobType::new("command.dashboard.update-dashboard");

pub struct UpdateDashboardJobInitializer {
    repo: DashboardRepo,
}

impl UpdateDashboardJobInitializer {
    pub fn new(repo: DashboardRepo) -> Self {
        Self { repo }
    }
}

impl JobInitializer for UpdateDashboardJobInitializer {
    type Config = UpdateDashboardConfig;

    fn job_type(&self) -> JobType {
        UPDATE_DASHBOARD_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateDashboardJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

pub struct UpdateDashboardJobRunner {
    config: UpdateDashboardConfig,
    repo: DashboardRepo,
}

#[async_trait]
impl JobRunner for UpdateDashboardJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "dashboard.update_dashboard_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing_utils::persistence::set_parent(&self.config.trace_context);

        let mut op = current_job.begin_op().await?;
        let mut dashboard = self.repo.load().await?;

        match &self.config.update {
            UpdateDashboardUpdate::FacilityProposalCreated { recorded_at } => {
                dashboard.last_updated = *recorded_at;
                dashboard.pending_facilities += 1;
            }
            UpdateDashboardUpdate::FacilityActivated { recorded_at } => {
                dashboard.last_updated = *recorded_at;
                dashboard.pending_facilities -= 1;
                dashboard.active_facilities += 1;
            }
            UpdateDashboardUpdate::FacilityCompleted { recorded_at } => {
                dashboard.last_updated = *recorded_at;
                dashboard.active_facilities -= 1;
            }
            UpdateDashboardUpdate::DisbursalSettled {
                recorded_at,
                amount,
            } => {
                dashboard.last_updated = *recorded_at;
                dashboard.total_disbursed += *amount;
            }
            UpdateDashboardUpdate::PaymentAllocationCreated {
                recorded_at,
                amount,
                obligation_type,
            } => {
                dashboard.last_updated = *recorded_at;
                if *obligation_type == ObligationType::Disbursal {
                    dashboard.total_disbursed -= *amount;
                }
            }
            UpdateDashboardUpdate::CollateralUpdated {
                recorded_at,
                direction,
                abs_diff,
            } => {
                dashboard.last_updated = *recorded_at;
                match direction {
                    CollateralDirection::Add => dashboard.total_collateral += *abs_diff,
                    CollateralDirection::Remove => dashboard.total_collateral -= *abs_diff,
                }
            }
        }

        self.repo.persist_in_tx(op.tx_mut(), &dashboard).await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
