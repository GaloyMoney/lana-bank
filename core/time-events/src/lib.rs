#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
mod closing_schedule;
mod config;
pub mod error;
mod event;
mod jobs;

use obix::{Outbox, out::OutboxEventMarker};
use tracing_macros::record_error_severity;

use domain_config::ExposedDomainConfigsReadOnly;

use crate::{
    closing_schedule::*,
    error::TimeEventsError,
    jobs::end_of_day::{EndOfDayProducerJobConfig, EndOfDayProducerJobInit},
};

pub use event::*;

#[derive(Clone)]
pub struct TimeEvents {
    _phantom: std::marker::PhantomData<()>,
}

impl TimeEvents {
    #[record_error_severity]
    #[tracing::instrument(name = "core_time_events.init", skip_all)]
    pub async fn init<E>(
        domain_configs: &ExposedDomainConfigsReadOnly,
        jobs: &mut job::Jobs,
        outbox: &Outbox<E>,
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
            _phantom: std::marker::PhantomData,
        })
    }
}
