#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
mod closing_schedule;
pub mod config;
pub mod error;
mod event;
mod jobs;

use audit::AuditSvc;
use authz::PermissionCheck;
use chrono::{DateTime, NaiveDate, Utc};
use domain_config::{DomainConfigAction, DomainConfigObject, ExposedDomainConfigsReadOnly};
use es_entity::clock::ClockHandle;
use obix::{Outbox, out::OutboxEventMarker};
use tracing_macros::record_error_severity;

use crate::{
    config::{ClosingTime, Timezone},
    error::TimeEventsError,
    jobs::end_of_day::{EndOfDayProducerJobConfig, EndOfDayProducerJobInit},
};

pub use closing_schedule::ClosingSchedule;
pub use event::*;

#[derive(Clone, Debug)]
pub struct TimeState {
    pub current_date: NaiveDate,
    pub current_time: DateTime<Utc>,
    pub next_end_of_day_at: DateTime<Utc>,
    pub can_advance_to_next_end_of_day: bool,
}

#[derive(Clone)]
pub struct TimeEvents<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    clock: ClockHandle,
    has_clock_controller: bool,
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
        has_clock_controller: bool,
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
            has_clock_controller,
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
            can_advance_to_next_end_of_day: self.has_clock_controller,
        })
    }
}
