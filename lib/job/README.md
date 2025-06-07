# Job

A small crate that provides a persistent job scheduling and execution system.
Jobs are stored in Postgres and executed by a polling executor.

## Basic usage

```rust
use job::{Jobs, JobExecutorConfig};

let config = JobExecutorConfig::default();
let mut jobs = Jobs::new(&pool, config);

// register job initializers
jobs.add_initializer(MyJobInitializer::new());

// start background polling
jobs.start_poll().await?;
```

To create a job you typically call `create_and_spawn_in_op` from within a
transaction:

```rust
let mut db = jobs.begin_op().await?;
let job = jobs
    .create_and_spawn_in_op(&mut db, JobId::new(), MyJobConfig { .. })
    .await?;
```

### Implementing a job

Jobs are described by a `JobInitializer` that constructs a `JobRunner` for a
specific `JobType`.

```rust
pub struct MyJobInitializer;

impl JobInitializer for MyJobInitializer {
    fn job_type() -> JobType { JobType::new("my-job") }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(MyJobRunner { config: job.config()? }))
    }
}
```

The `JobRunner` implements the actual work and returns [`JobCompletion`] to
control whether the job completes or should be rescheduled.

```rust
#[async_trait]
impl JobRunner for MyJobRunner {
    async fn run(&self, current: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // perform work using `current`
        Ok(JobCompletion::Complete)
    }
}
```

## Configuration

The executor is configured via [`JobExecutorConfig`]:

- `poll_interval` – how often to poll for new jobs (default: 5 seconds)
- `max_jobs_per_process` – maximum concurrent jobs (default: 20)
- `min_jobs_per_process` – when concurrency drops below this, new jobs are polled (default: 15)

`JobExecutorConfig` implements `Default` and `serde::Deserialize`, so it can be
loaded from configuration files.
