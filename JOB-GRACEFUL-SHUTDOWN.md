# Job Graceful Shutdown - Usage Guide

## Overview

The job library now supports graceful shutdown, allowing jobs to detect shutdown requests and complete their work cleanly instead of being abruptly aborted.

## Key Changes

### 1. CurrentJob API

The `CurrentJob` struct now provides two methods for detecting shutdown:

```rust
// Async method - waits for shutdown signal
pub async fn shutdown_requested(&mut self) -> bool

// Non-blocking method - checks if shutdown was requested
pub fn is_shutdown_requested(&mut self) -> bool
```

### 2. Shutdown Behavior

- When `Jobs::shutdown()` is called, a broadcast signal is sent to all running jobs
- The system waits up to `shutdown_timeout` (default: 30 seconds) for jobs to complete
- After timeout, remaining jobs are rescheduled as "pending" in the database
- Jobs that don't check for shutdown continue running until timeout

## Usage Patterns

### Pattern 1: Continuous Loop Jobs

For jobs that run in a continuous loop (e.g., polling jobs):

```rust
#[async_trait]
impl JobRunner for MyJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,  // Note: changed from _current_job to mut current_job
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            // Do some work
            let result = do_work().await?;
            
            // Use tokio::select to wait for either sleep or shutdown
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(60)) => {
                    // Normal sleep completed, continue loop
                }
                _ = current_job.shutdown_requested() => {
                    // Shutdown requested, exit gracefully
                    tracing::info!("Job received shutdown signal, exiting gracefully");
                    return Ok(JobCompletion::Complete);
                }
            }
        }
    }
}
```

**Example:** `core/price/src/jobs/get_price_from_bfx.rs`

### Pattern 2: Polling Check in Loop

For jobs that process items in batches:

```rust
#[async_trait]
impl JobRunner for MyJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        loop {
            // Check for shutdown before processing next batch
            if current_job.is_shutdown_requested() {
                tracing::info!("Shutdown requested, finishing current batch and exiting");
                return Ok(JobCompletion::Complete);
            }
            
            // Process a batch of items
            let items = fetch_items().await?;
            for item in items {
                process_item(item).await?;
            }
            
            // Small delay between batches
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
```

### Pattern 3: Long-Running Single Task

For jobs that do one long operation:

```rust
#[async_trait]
impl JobRunner for MyJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Use tokio::select for cancellable operations
        tokio::select! {
            result = expensive_operation() => {
                result?;
                Ok(JobCompletion::Complete)
            }
            _ = current_job.shutdown_requested() => {
                tracing::warn!("Shutdown during operation, will reschedule");
                // Job will be rescheduled automatically
                Err("Interrupted by shutdown".into())
            }
        }
    }
}
```

### Pattern 4: Jobs That Already Reschedule

For jobs that naturally complete and reschedule, no changes needed:

```rust
#[async_trait]
impl JobRunner for MyJobRunner {
    async fn run(
        &self,
        current_job: CurrentJob,  // Can still use non-mut if not checking shutdown
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Do work
        process_items().await?;
        
        // Reschedule for later
        Ok(JobCompletion::RescheduleIn(Duration::from_secs(60)))
    }
}
```

These jobs will be interrupted by timeout if they run too long, but will be rescheduled automatically.

## Configuration

You can adjust the shutdown timeout in your job poller config:

```rust
let config = JobSvcConfig::builder()
    .pool(pool)
    .poller_config(JobPollerConfig {
        shutdown_timeout: Duration::from_secs(60), // Wait up to 60 seconds
        ..Default::default()
    })
    .build()?;
```

## Testing

### Manual Test

Run the test script:

```bash
./test-job-shutdown.sh
```

Then press Ctrl+C to trigger shutdown and observe the logs.

### Expected Log Output

```
INFO Sending shutdown signal to servers
INFO Shutting down tracer provider
INFO n_jobs=1 timeout_secs=30 Waiting for jobs to complete
INFO Price fetch job received shutdown signal, exiting gracefully
INFO All jobs completed gracefully
INFO Server handles finished
```

## Migration Guide

To update existing jobs:

1. Change `_current_job: CurrentJob` to `mut current_job: CurrentJob` in the `run` method signature
2. Add shutdown detection using one of the patterns above
3. Return `JobCompletion::Complete` when shutdown is detected
4. Add logging to indicate graceful shutdown

## Benefits

- **Cleaner shutdown**: Jobs can finish their current work
- **Data consistency**: No mid-operation interruptions
- **Better observability**: Log when jobs exit due to shutdown
- **Configurable**: Adjust timeout based on your needs
- **Backward compatible**: Jobs without shutdown checks still work

