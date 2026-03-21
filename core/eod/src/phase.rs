use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    credit_facility_eod_process::{
        CreditFacilityEodProcessConfig, CreditFacilityEodProcessSpawner,
    },
    deposit_activity_process::{DepositActivityProcessConfig, DepositActivityProcessSpawner},
    obligation_status_process::{ObligationStatusProcessConfig, ObligationStatusProcessSpawner},
};

/// Context passed to each phase when spawning its job.
#[derive(Clone)]
pub struct EodContext {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
}

/// A registered EOD phase. Each product implements this to plug into the EOD pipeline.
/// The trait erases the typed `JobSpawner<C>` behind a trait object.
#[async_trait]
pub trait EodPhase: Send + Sync {
    /// Unique name for this phase (e.g. "obligation-status", "deposit-activity").
    fn name(&self) -> &str;

    /// Spawn the phase child job inside the given DB operation.
    async fn spawn_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        job_id: job::JobId,
        ctx: &EodContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Adapter for the obligation-status child process.
pub struct ObligationStatusPhase {
    spawner: ObligationStatusProcessSpawner,
}

impl ObligationStatusPhase {
    pub fn new(spawner: ObligationStatusProcessSpawner) -> Self {
        Self { spawner }
    }
}

#[async_trait]
impl EodPhase for ObligationStatusPhase {
    fn name(&self) -> &str {
        "obligation-status"
    }

    async fn spawn_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        job_id: job::JobId,
        ctx: &EodContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.spawner
            .spawn_in_op(op, job_id, ObligationStatusProcessConfig { date: ctx.date })
            .await?;
        Ok(())
    }
}

/// Adapter for the deposit-activity child process.
pub struct DepositActivityPhase {
    spawner: DepositActivityProcessSpawner,
}

impl DepositActivityPhase {
    pub fn new(spawner: DepositActivityProcessSpawner) -> Self {
        Self { spawner }
    }
}

#[async_trait]
impl EodPhase for DepositActivityPhase {
    fn name(&self) -> &str {
        "deposit-activity"
    }

    async fn spawn_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        job_id: job::JobId,
        ctx: &EodContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.spawner
            .spawn_in_op(
                op,
                job_id,
                DepositActivityProcessConfig {
                    date: ctx.date,
                    closing_time: ctx.closing_time,
                },
            )
            .await?;
        Ok(())
    }
}

/// Adapter for the credit-facility EOD child process.
pub struct CreditFacilityEodPhase {
    spawner: CreditFacilityEodProcessSpawner,
}

impl CreditFacilityEodPhase {
    pub fn new(spawner: CreditFacilityEodProcessSpawner) -> Self {
        Self { spawner }
    }
}

#[async_trait]
impl EodPhase for CreditFacilityEodPhase {
    fn name(&self) -> &str {
        "credit-facility-eod"
    }

    async fn spawn_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        job_id: job::JobId,
        ctx: &EodContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.spawner
            .spawn_in_op(
                op,
                job_id,
                CreditFacilityEodProcessConfig { date: ctx.date },
            )
            .await?;
        Ok(())
    }
}
