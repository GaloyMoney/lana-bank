use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use job::*;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{DomainConfigCache, public::CoreDomainConfigEvent};

#[derive(Serialize, Deserialize)]
pub struct DomainConfigCacheInvalidationJobConfig;

pub struct DomainConfigCacheInvalidationInit<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent> + Send + Sync + 'static,
{
    outbox: Outbox<E>,
    cache: DomainConfigCache,
}

impl<E> DomainConfigCacheInvalidationInit<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent> + Send + Sync + 'static,
{
    pub fn new(outbox: &Outbox<E>, cache: &DomainConfigCache) -> Self {
        Self {
            outbox: outbox.clone(),
            cache: cache.clone(),
        }
    }
}

const DOMAIN_CONFIG_CACHE_INVALIDATION_JOB: JobType =
    JobType::new("outbox.domain-config-cache-invalidation");

impl<E> JobInitializer for DomainConfigCacheInvalidationInit<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent> + Send + Sync + 'static,
{
    type Config = DomainConfigCacheInvalidationJobConfig;

    fn job_type(&self) -> JobType {
        DOMAIN_CONFIG_CACHE_INVALIDATION_JOB
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DomainConfigCacheInvalidationJobRunner {
            outbox: self.outbox.clone(),
            cache: self.cache.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct DomainConfigCacheInvalidationJobData {
    sequence: obix::EventSequence,
}

pub struct DomainConfigCacheInvalidationJobRunner<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent>,
{
    outbox: Outbox<E>,
    cache: DomainConfigCache,
}

impl<E> DomainConfigCacheInvalidationJobRunner<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent>,
{
    #[instrument(
        name = "domain_config.cache_invalidation_job.process_message",
        parent = None,
        skip(self, message),
        fields(seq = %message.sequence, handled = false)
    )]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(payload) = &message.payload
            && let Some(event) = payload.as_event()
        {
            match event {
                CoreDomainConfigEvent::DomainConfigUpdated { key } => {
                    message.inject_trace_parent();
                    Span::current().record("handled", true);
                    tracing::info!(key = %key, "Invalidating cache for domain config");
                    self.cache.invalidate(key).await;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<E> JobRunner for DomainConfigCacheInvalidationJobRunner<E>
where
    E: OutboxEventMarker<CoreDomainConfigEvent> + Send + Sync + 'static,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<DomainConfigCacheInvalidationJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %DOMAIN_CONFIG_CACHE_INVALIDATION_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            self.process_message(&message).await?;
                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state(&state)
                                .await?;
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
