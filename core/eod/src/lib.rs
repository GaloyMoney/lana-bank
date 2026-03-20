#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod accrue_interest_command;
pub mod complete_accrual_cycle_command;
pub mod credit_facility_eod_process;
pub mod deposit_activity_process;
mod end_of_day_handler;
pub mod eod_process;
pub mod error;
mod event;
pub mod interest_accrual_process;
pub mod obligation_status_process;
mod primitives;
mod process_manager;
pub mod public;
mod publisher;

use es_entity::clock::ClockHandle;
use obix::out::{OutboxEventJobConfig, OutboxEventMarker};
use sqlx::PgPool;

use crate::{
    end_of_day_handler::{END_OF_DAY_HANDLER_JOB, EndOfDayHandler},
    error::CoreEodError,
};

pub use eod_process::{EodProcess, EodProcessEvent, EodProcesses, NewEodProcess};
pub use event::*;
pub use primitives::*;
pub use process_manager::{
    EOD_PROCESS_MANAGER_JOB, EodProcessManagerConfig, EodProcessManagerJobInit,
    EodProcessManagerJobSpawner,
};
pub use public::*;
pub use publisher::EodPublisher;

/// The core-eod module: owns EOD process orchestration.
///
/// Listens for `CoreTimeEvent::EndOfDay` via the outbox and spawns the
/// `EodProcessManager` job, which then orchestrates all EOD phases.
pub struct CoreEod<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    eod_processes: EodProcesses<E>,
}

impl<E> Clone for CoreEod<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    fn clone(&self) -> Self {
        Self {
            eod_processes: self.eod_processes.clone(),
        }
    }
}

impl<E> CoreEod<E>
where
    E: OutboxEventMarker<CoreEodEvent>,
{
    /// Initialize the EOD module.
    ///
    /// - Registers the `EndOfDayHandler` outbox listener for `CoreTimeEvent::EndOfDay`
    /// - Registers the `EodProcessManager` job initializer
    pub async fn init<TE>(
        pool: &PgPool,
        jobs: &mut job::Jobs,
        outbox: &obix::Outbox<E>,
        time_outbox: &obix::Outbox<TE>,
        clock: ClockHandle,
        obligation_status_process_spawner: obligation_status_process::ObligationStatusProcessSpawner,
        deposit_activity_process_spawner: deposit_activity_process::DepositActivityProcessSpawner,
        credit_facility_eod_process_spawner: credit_facility_eod_process::CreditFacilityEodProcessSpawner,
    ) -> Result<Self, CoreEodError>
    where
        TE: OutboxEventMarker<core_time_events::CoreTimeEvent>,
    {
        let publisher = EodPublisher::new(outbox);
        let eod_processes = EodProcesses::new(pool, &publisher, clock);

        let eod_pm_spawner = jobs.add_initializer(EodProcessManagerJobInit::new(
            jobs,
            eod_processes.clone(),
            obligation_status_process_spawner,
            deposit_activity_process_spawner,
            credit_facility_eod_process_spawner,
        ));

        time_outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(END_OF_DAY_HANDLER_JOB),
                EndOfDayHandler::new(&eod_pm_spawner),
            )
            .await?;

        Ok(Self { eod_processes })
    }

    /// Find the latest EOD process, if any.
    pub async fn find_latest_process(
        &self,
    ) -> Result<Option<EodProcess>, eod_process::error::EodProcessError> {
        self.eod_processes.find_latest().await
    }

    /// Get the status of the latest EOD process, if any.
    pub async fn latest_eod_status(
        &self,
    ) -> Result<Option<EodProcessStatus>, eod_process::error::EodProcessError> {
        Ok(self.eod_processes.find_latest().await?.map(|p| p.status()))
    }
}
