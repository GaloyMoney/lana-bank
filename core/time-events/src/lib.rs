#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
mod closing_schedule;
pub mod config;
pub mod error;
mod event;
mod jobs;

// --- Modules merged from core-eod ---
pub mod accrue_interest_command;
pub mod complete_accrual_cycle_command;
pub mod credit_facility_eod_process;
pub mod deposit_activity_process;
pub mod end_of_day_handler;
pub mod eod_process;
pub mod interest_accrual_process;
mod job_id;
pub mod obligation_transition_process;
mod primitives;
mod process_manager;
pub mod public;
mod publisher;

use audit::AuditSvc;
use authz::PermissionCheck;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use domain_config::{DomainConfigAction, DomainConfigObject, ExposedDomainConfigsReadOnly};
use es_entity::clock::{ClockController, ClockHandle};
use obix::{Outbox, out::OutboxEventMarker};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing_macros::record_error_severity;

use crate::{
    config::{ClosingTime, Timezone},
    error::TimeEventsError,
    jobs::end_of_day::{EndOfDayProducerJobConfig, EndOfDayProducerJobInit},
};

pub use closing_schedule::ClosingSchedule;
pub use event::*;

// --- Re-exports merged from core-eod ---
pub use eod_process::{EodPhase, EodProcess, EodProcessEvent, EodProcesses, NewEodProcess};
pub use job_id::*;
pub use primitives::*;
pub use process_manager::{
    EOD_PROCESS_MANAGER_JOB_TYPE, EodProcessManagerConfig, EodProcessManagerJobInit,
    EodProcessManagerJobSpawner,
};
pub use public::*;
pub use publisher::EodPublisher;

#[derive(Clone, Debug)]
pub struct TimeState {
    pub current_date: NaiveDate,
    pub current_time: DateTime<Utc>,
    pub next_end_of_day_at: DateTime<Utc>,
    pub timezone: Tz,
    pub end_of_day_time: NaiveTime,
    pub can_advance_to_next_end_of_day: bool,
}

#[derive(Clone)]
pub struct TimeEvents<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    clock: ClockHandle,
    clock_controller: Option<ClockController>,
    manual_advance_guard: Arc<Mutex<()>>,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl<Perms> TimeEvents<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<DomainConfigObject>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "core_time_events.init", skip_all)]
    pub async fn init<E>(
        authz: &Perms,
        domain_configs: &ExposedDomainConfigsReadOnly,
        jobs: &mut job::Jobs,
        outbox: &Outbox<E>,
        clock: &ClockHandle,
        clock_controller: Option<ClockController>,
    ) -> Result<Self, TimeEventsError>
    where
        E: OutboxEventMarker<CoreTimeEvent>,
    {
        let end_of_day_producer_job_spawner =
            jobs.add_initializer(EndOfDayProducerJobInit::new(outbox, domain_configs));
        end_of_day_producer_job_spawner
            .spawn_unique(
                job::JobId::new(),
                EndOfDayProducerJobConfig {
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        Ok(Self {
            authz: authz.clone(),
            clock: clock.clone(),
            clock_controller,
            manual_advance_guard: Arc::new(Mutex::new(())),
            domain_configs: domain_configs.clone(),
        })
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core_time_events.state", skip(self))]
    pub async fn state(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<TimeState, TimeEventsError> {
        self.authz
            .enforce_permission(
                sub,
                DomainConfigObject::all_exposed_configs(),
                DomainConfigAction::EXPOSED_CONFIG_READ,
            )
            .await?;

        self.state_inner().await
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "core_time_events.advance_to_next_end_of_day",
        skip(self),
        fields(next_end_of_day_at = tracing::field::Empty)
    )]
    pub async fn advance_to_next_end_of_day(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<TimeState, TimeEventsError> {
        self.authz
            .enforce_permission(
                sub,
                DomainConfigObject::all_exposed_configs(),
                DomainConfigAction::EXPOSED_CONFIG_WRITE,
            )
            .await?;

        let Some(controller) = self.clock_controller.as_ref() else {
            return Err(TimeEventsError::TimeAdvanceUnavailable);
        };

        // Serialize manual clock advancement so concurrent requests cannot
        // compute the same target and double-advance the shared clock.
        // FIXME: The long-term fix is to reject another move-to-next-day request
        // until all processing for the current EOD has completed.
        let _manual_advance_guard = self.manual_advance_guard.lock().await;

        let before = self.state_inner().await?;
        tracing::Span::current().record(
            "next_end_of_day_at",
            tracing::field::display(before.next_end_of_day_at),
        );

        let advance_by = (before.next_end_of_day_at - before.current_time)
            .to_std()
            .map_err(|_| {
                TimeEventsError::TimeAdvanceFailed(
                    "Next end of day is not in the future".to_string(),
                )
            })?;

        let _ = controller.advance(advance_by).await;

        let after = self.state_inner().await?;
        if advance_by > Duration::ZERO && after.current_time < before.next_end_of_day_at {
            return Err(TimeEventsError::TimeAdvanceFailed(
                "Artificial clock is not manually advanceable".to_string(),
            ));
        }

        Ok(after)
    }

    /// Internal state query without authorization check.
    async fn state_inner(&self) -> Result<TimeState, TimeEventsError> {
        let current_time = self.clock.now();
        let timezone = self
            .domain_configs
            .get_without_audit::<Timezone>()
            .await?
            .value();
        let closing_time = self
            .domain_configs
            .get_without_audit::<ClosingTime>()
            .await?
            .value();
        let schedule = ClosingSchedule::from_time(timezone, closing_time, current_time);

        Ok(TimeState {
            current_date: schedule.current_day(),
            current_time,
            next_end_of_day_at: schedule.next_closing(),
            timezone,
            end_of_day_time: closing_time,
            can_advance_to_next_end_of_day: self.clock_controller.is_some(),
        })
    }
}
