# Error Handling: Permanent vs On-Demand Jobs

## Overview

The job system has fundamentally different error handling strategies for permanent jobs vs on-demand jobs, reflecting their different purposes and lifecycle expectations.

## Key Differences

### **Permanent Jobs (Event Processors)**

**Retry Strategy:**
```rust
fn retry_on_error_settings() -> RetrySettings {
    RetrySettings::repeat_indefinitely()
}
```

**Characteristics:**
- **Infinite Retries**: `n_attempts: None` - will retry forever
- **No Warnings**: `n_warn_attempts: None` - errors logged as INFO/WARN
- **Always Reschedule**: Return `JobCompletion::RescheduleNow` 
- **Never Complete**: Jobs run indefinitely until application shutdown

**Rationale:**
- These are **critical system infrastructure** that must never stop
- Event processing **must eventually succeed** for data consistency
- Temporary failures (network, database locks) should be retried indefinitely
- System health depends on these jobs continuing to run

### **On-Demand Jobs (Finite Tasks)**

**Retry Strategy:**
```rust
fn retry_on_error_settings() -> RetrySettings {
    Default::default()  // Limited retries
}
```

**Default Settings:**
```rust
RetrySettings {
    n_attempts: Some(30),           // Give up after 30 attempts
    n_warn_attempts: Some(3),       // Warn after 3 failed attempts  
    min_backoff: Duration::from_secs(1),
    max_backoff: Duration::from_secs(60 * 60 * 24 * 30), // 1 month
    backoff_jitter_pct: 20,
}
```

**Characteristics:**
- **Limited Retries**: Maximum 30 attempts before giving up
- **Escalating Warnings**: First 3 failures logged as WARN, then ERROR
- **Exponential Backoff**: 1s → 2s → 4s → ... → 30 days (capped)
- **Eventually Complete**: Either succeed with `JobCompletion::Complete` or fail permanently

**Rationale:**
- These handle **specific business tasks** that can fail permanently
- Failed tasks should be investigated rather than retried forever
- Resource protection: don't consume infinite CPU/memory on hopeless tasks
- Business impact: some operations genuinely cannot be completed

## Error Handling Flow

### **When Jobs Fail**

The `fail_job` function in executor handles failures:

```rust
async fn fail_job(
    mut op: es_entity::DbOp<'_>,
    id: JobId,
    attempt: u32,
    error: JobError,
    repo: JobRepo,
    retry_settings: &RetrySettings,
) -> Result<(), JobError> {
    let mut job = repo.find_by_id(id).await?;
    job.fail(error.to_string());  // Record error in job entity
    repo.update_in_op(&mut op, &mut job).await?;

    if retry_settings.n_attempts.unwrap_or(u32::MAX) > attempt {
        // Reschedule for retry with exponential backoff
        let reschedule_at = retry_settings.next_attempt_at(attempt);
        sqlx::query!(
            "UPDATE job_executions SET state = 'pending', reschedule_after = $2, attempt_index = $3 WHERE id = $1",
            id as JobId, reschedule_at, (attempt + 1) as i32
        ).execute(&mut **op.tx()).await?;
    } else {
        // Permanent failure - delete execution and stop retrying
        sqlx::query!("DELETE FROM job_executions WHERE id = $1", id as JobId)
            .execute(&mut **op.tx()).await?;
    }
}
```

### **Permanent Jobs**: Infinite Loop
1. Job fails → `n_attempts = None` → Always retry
2. Reschedule with exponential backoff (1s → 2s → 4s → ... → 30 days)
3. Eventually retries when external issue resolves
4. **Never deleted** from job_executions table

### **On-Demand Jobs**: Finite Attempts  
1. Job fails → Increment attempt counter
2. If `attempt < 30` → Reschedule with backoff
3. If `attempt >= 30` → **Permanent failure**
4. **Deleted** from job_executions table (cannot retry)
5. Job entity status becomes `JobStatus::Errored`

## Error Logging Levels

The system uses different log levels based on attempt count:

```rust
let n_warn_attempts = retry_settings.n_warn_attempts;
if attempt <= n_warn_attempts.unwrap_or(u32::MAX) {
    // Log as WARN level  
} else {
    // Log as ERROR level
}
```

**Permanent Jobs:**
- `n_warn_attempts = None` → Always log as WARN
- Prevents log spam from expected retries

**On-Demand Jobs:**
- First 3 attempts → WARN level
- Attempts 4-30 → ERROR level
- Escalation signals increasing concern

## Job Completion Patterns

### **Permanent Jobs Always Loop:**
```rust
async fn run(&self, current_job: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
    let mut state = current_job.execution_state::<JobData>()?;
    let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;
    
    while let Some(message) = stream.next().await {
        // Process events...
        state.sequence = message.sequence;
        current_job.update_execution_state(state).await?;
    }
    
    Ok(JobCompletion::RescheduleNow)  // ← Always reschedule immediately
}
```

### **On-Demand Jobs Finish:**
```rust
async fn run(&self, current_job: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
    let config = current_job.config::<MyJobConfig>()?;
    
    // Do specific work for specific entity
    self.process_obligation(config.obligation_id).await?;
    
    Ok(JobCompletion::Complete)  // ← Finish and delete
}
```

## Summary

| Aspect | Permanent Jobs | On-Demand Jobs |
|--------|----------------|----------------|
| **Retry Count** | Infinite (`None`) | Limited (30) |
| **Error Escalation** | Always WARN | WARN → ERROR |
| **Backoff** | Exponential | Exponential |
| **Max Backoff** | 30 days | 30 days |
| **Completion** | Never (`RescheduleNow`) | Always (`Complete`) |
| **Failure Outcome** | Keep retrying | Permanent failure |
| **Purpose** | Infrastructure reliability | Business task completion |
| **Examples** | Event processors, sync jobs | Reports, notifications, deadlines |

The fundamental difference is **permanence**: permanent jobs are infrastructure that must never stop, while on-demand jobs are business tasks that can legitimately fail and need human intervention.