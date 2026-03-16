#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
mod closing_schedule;
pub mod config;
pub mod error;
mod event;
mod jobs;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use domain_config::ExposedDomainConfigsReadOnly;
use es_entity::clock::{ClockController, ClockHandle};
use obix::{Outbox, out::OutboxEventMarker};
use std::time::Duration;
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
    pub timezone: Tz,
    pub end_of_day_time: NaiveTime,
    pub can_advance_to_next_end_of_day: bool,
}

#[derive(Clone)]
pub struct TimeEvents {
    clock: ClockHandle,
    clock_controller: Option<ClockController>,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl TimeEvents {
    #[record_error_severity]
    #[tracing::instrument(name = "core_time_events.init", skip_all)]
    pub async fn init<E>(
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
            clock: clock.clone(),
            clock_controller,
            domain_configs: domain_configs.clone(),
        })
    }

    #[record_error_severity]
    #[tracing::instrument(name = "core_time_events.state", skip(self))]
    pub async fn state(&self) -> Result<TimeState, TimeEventsError> {
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

    #[record_error_severity]
    #[tracing::instrument(
        name = "core_time_events.advance_to_next_end_of_day",
        skip(self),
        fields(next_end_of_day_at = tracing::field::Empty)
    )]
    pub async fn advance_to_next_end_of_day(&self) -> Result<TimeState, TimeEventsError> {
        let Some(controller) = self.clock_controller.as_ref() else {
            return Err(TimeEventsError::TimeAdvanceUnavailable);
        };

        let before = self.state().await?;
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

        let after = self.state().await?;
        if advance_by > Duration::ZERO && after.current_time < before.next_end_of_day_at {
            return Err(TimeEventsError::TimeAdvanceFailed(
                "Artificial clock is not manually advanceable".to_string(),
            ));
        }

        Ok(after)
    }
}
