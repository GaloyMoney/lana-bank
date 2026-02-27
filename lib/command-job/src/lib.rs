#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use std::sync::Arc;

use async_trait::async_trait;

use serde::{Serialize, de::DeserializeOwned};

use job::*;

/// Trait for one-time command jobs spawned by event handlers.
///
/// Reduces the boilerplate of implementing separate `JobInitializer` + `JobRunner`
/// pairs by collapsing them into a single trait implementation.
///
/// # Return type
///
/// `run` returns `Result<JobCompletion, Box<dyn Error>>`:
/// - `Ok(JobCompletion::Complete)` — success
/// - `Ok(JobCompletion::RescheduleIn(duration))` — explicit retry with custom delay
/// - `Ok(JobCompletion::RescheduleNow)` — immediate reschedule
/// - `Err(e)` — retryable error (uses `retry_settings()` backoff)
#[async_trait]
pub trait CommandJob: Send + Sync + 'static {
    type Command: Serialize + DeserializeOwned + Clone + Send + Sync + 'static;

    fn job_type() -> JobType;

    fn entity_id(command: &Self::Command) -> String;

    fn retry_settings() -> RetrySettings {
        RetrySettings::default()
    }

    async fn run(
        &self,
        current_job: CurrentJob,
        command: &Self::Command,
    ) -> Result<JobCompletion, Box<dyn std::error::Error + Send + Sync>>;
}

/// Registers a `CommandJob` with the job system and returns a type-erased spawner.
///
/// This is the primary entry point for wiring command jobs. It combines
/// `CommandJobInitializer` construction, job registration, and spawner creation
/// into a single call. The returned `CommandJobSpawner` is generic only over
/// the `Command` type, not the full `CommandJob` — so handlers stay non-generic.
pub fn build_command_job<C: CommandJob>(
    jobs: &mut Jobs,
    command_job: C,
) -> CommandJobSpawner<C::Command> {
    let spawner = jobs.add_initializer(CommandJobInitializer::new(command_job));
    CommandJobSpawner {
        inner: spawner,
        entity_id_fn: C::entity_id,
    }
}

/// Bridges a `CommandJob` implementation to the `JobInitializer` trait.
pub(crate) struct CommandJobInitializer<C: CommandJob> {
    command_job: Arc<C>,
}

impl<C: CommandJob> CommandJobInitializer<C> {
    fn new(command_job: C) -> Self {
        Self {
            command_job: Arc::new(command_job),
        }
    }
}

impl<C: CommandJob> JobInitializer for CommandJobInitializer<C> {
    type Config = C::Command;

    fn job_type(&self) -> JobType {
        C::job_type()
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        C::retry_settings()
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let command: C::Command = job.config()?;
        Ok(Box::new(CommandJobRunner {
            command_job: self.command_job.clone(),
            command,
        }))
    }
}

struct CommandJobRunner<C: CommandJob> {
    command_job: Arc<C>,
    command: C::Command,
}

#[async_trait]
impl<C: CommandJob> JobRunner for CommandJobRunner<C> {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if current_job.is_shutdown_requested() {
            return Ok(JobCompletion::RescheduleNow);
        }
        self.command_job
            .run(current_job, &self.command)
            .await
            .map_err(|e| e as Box<dyn std::error::Error>)
    }
}

/// Wraps `JobSpawner` to provide entity-id-based queue control.
///
/// Generic over the `Command` type only (not the full `CommandJob`), so handlers
/// that hold this spawner don't need to propagate the job's generic parameters.
pub struct CommandJobSpawner<Command>
where
    Command: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    inner: JobSpawner<Command>,
    entity_id_fn: fn(&Command) -> String,
}

impl<Command> Clone for CommandJobSpawner<Command>
where
    Command: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            entity_id_fn: self.entity_id_fn,
        }
    }
}

impl<Command> CommandJobSpawner<Command>
where
    Command: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn spawn(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        command: Command,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let queue_id = (self.entity_id_fn)(&command);
        self.inner
            .spawn_with_queue_id_in_op(op, JobId::new(), command, queue_id)
            .await?;
        Ok(())
    }
}

/// Trait for command jobs where all work happens inside a single database transaction.
///
/// The framework opens the transaction (`current_job.begin_op()`), passes it to `run`,
/// and on success returns `JobCompletion::CompleteWithOp(op)` to commit atomically.
/// On error, the transaction is dropped (rolled back) and the error propagates for retry.
///
/// Use this for jobs that only do database work. For jobs that call external services
/// or need custom scheduling, use [`CommandJob`] instead.
#[async_trait]
pub trait AtomicCommandJob: Send + Sync + 'static {
    type Command: Serialize + DeserializeOwned + Clone + Send + Sync + 'static;

    fn job_type() -> JobType;

    fn entity_id(command: &Self::Command) -> String;

    fn retry_settings() -> RetrySettings {
        RetrySettings::default()
    }

    async fn run(
        &self,
        op: &mut es_entity::DbOp<'static>,
        command: &Self::Command,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Registers an `AtomicCommandJob` with the job system and returns a type-erased spawner.
pub fn build_atomic_command_job<C: AtomicCommandJob>(
    jobs: &mut Jobs,
    command_job: C,
) -> CommandJobSpawner<C::Command> {
    let spawner = jobs.add_initializer(AtomicCommandJobInitializer::new(command_job));
    CommandJobSpawner {
        inner: spawner,
        entity_id_fn: C::entity_id,
    }
}

pub(crate) struct AtomicCommandJobInitializer<C: AtomicCommandJob> {
    command_job: Arc<C>,
}

impl<C: AtomicCommandJob> AtomicCommandJobInitializer<C> {
    fn new(command_job: C) -> Self {
        Self {
            command_job: Arc::new(command_job),
        }
    }
}

impl<C: AtomicCommandJob> JobInitializer for AtomicCommandJobInitializer<C> {
    type Config = C::Command;

    fn job_type(&self) -> JobType {
        C::job_type()
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        C::retry_settings()
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let command: C::Command = job.config()?;
        Ok(Box::new(AtomicCommandJobRunner {
            command_job: self.command_job.clone(),
            command,
        }))
    }
}

struct AtomicCommandJobRunner<C: AtomicCommandJob> {
    command_job: Arc<C>,
    command: C::Command,
}

#[async_trait]
impl<C: AtomicCommandJob> JobRunner for AtomicCommandJobRunner<C> {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if current_job.is_shutdown_requested() {
            return Ok(JobCompletion::RescheduleNow);
        }
        let mut op = current_job.begin_op().await?;
        self.command_job
            .run(&mut op, &self.command)
            .await
            .map_err(|e| e as Box<dyn std::error::Error>)?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
