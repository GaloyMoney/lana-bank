---
id: background-jobs
title: Background Jobs
sidebar_position: 6
---

# Background Jobs System

This document describes the background job processing system used for asynchronous operations.

![Background Jobs Architecture](/img/architecture/background-jobs-1.png)

## Overview

Lana uses a job system for:

- Asynchronous processing
- Scheduled tasks
- Retryable operations
- Cross-service coordination

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    JOB SYSTEM                                   │
│                                                                  │
│  ┌─────────────────┐                                            │
│  │   Job Creator   │                                            │
│  │  (Domain Service)│                                           │
│  └─────────────────┘                                            │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Job Queue                             │   │
│  │                 (PostgreSQL Table)                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │  Job Dispatcher │───▶│   Job Executor  │                    │
│  │                 │    │                 │                    │
│  └─────────────────┘    └─────────────────┘                    │
│                                │                                │
│                                ▼                                │
│                        ┌──────────────┐                        │
│                        │  Job Result  │                        │
│                        │ (Success/Fail)│                       │
│                        └──────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

## Job Types

| Job Type | Purpose | Example |
|----------|---------|---------|
| Approval Processing | Execute governance decisions | Approve disbursal |
| Interest Accrual | Calculate periodic interest | Daily interest |
| Notifications | Send alerts and emails | Payment reminder |
| Sync | External system synchronization | Portfolio valuation |

## Job Definition

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    id: JobId,
    job_type: JobType,
    payload: serde_json::Value,
    status: JobStatus,
    attempts: u32,
    max_attempts: u32,
    scheduled_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}
```

## Job Execution

### Job Tracker

Manages job lifecycle:

```rust
pub struct JobTracker {
    pool: PgPool,
}

impl JobTracker {
    pub async fn enqueue(&self, job: NewJob) -> Result<JobId> {
        // Insert job into queue
    }

    pub async fn fetch_ready(&self, limit: u32) -> Result<Vec<Job>> {
        // Get jobs ready for execution
    }

    pub async fn mark_completed(&self, id: JobId) -> Result<()> {
        // Mark job as completed
    }

    pub async fn mark_failed(&self, id: JobId, error: String) -> Result<()> {
        // Mark job as failed, possibly schedule retry
    }
}
```

### Job Dispatcher

Executes jobs based on type:

```rust
pub struct JobDispatcher {
    executors: HashMap<JobType, Box<dyn JobExecutor>>,
}

impl JobDispatcher {
    pub async fn dispatch(&self, job: Job) -> Result<JobResult> {
        let executor = self.executors
            .get(&job.job_type)
            .ok_or(Error::UnknownJobType)?;

        executor.execute(job.payload).await
    }
}
```

## Retry Logic

Failed jobs are retried with exponential backoff:

```rust
impl Job {
    pub fn calculate_next_retry(&self) -> DateTime<Utc> {
        let delay_seconds = 2u64.pow(self.attempts) * 60;
        Utc::now() + Duration::seconds(delay_seconds as i64)
    }

    pub fn should_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }
}
```

### Retry Configuration

| Attempt | Delay |
|---------|-------|
| 1 | 2 minutes |
| 2 | 4 minutes |
| 3 | 8 minutes |
| 4 | 16 minutes |
| 5 | 32 minutes (max) |

## Scheduled Jobs

Jobs can be scheduled for future execution:

```rust
// Schedule interest accrual for midnight
let job = NewJob {
    job_type: JobType::InterestAccrual,
    payload: json!({}),
    scheduled_at: next_midnight(),
};

tracker.enqueue(job).await?;
```

## Job Examples

### Approval Processing Job

```rust
pub struct ApprovalProcessingExecutor {
    governance: GovernanceService,
}

impl JobExecutor for ApprovalProcessingExecutor {
    async fn execute(&self, payload: Value) -> Result<JobResult> {
        let input: ApprovalInput = serde_json::from_value(payload)?;

        self.governance
            .process_approval(input.process_id)
            .await?;

        Ok(JobResult::Success)
    }
}
```

### Interest Accrual Job

```rust
pub struct InterestAccrualExecutor {
    credit_service: CreditService,
}

impl JobExecutor for InterestAccrualExecutor {
    async fn execute(&self, payload: Value) -> Result<JobResult> {
        let facilities = self.credit_service
            .get_active_facilities()
            .await?;

        for facility in facilities {
            self.credit_service
                .accrue_interest(facility.id)
                .await?;
        }

        Ok(JobResult::Success)
    }
}
```

## Monitoring

### Metrics

- Jobs enqueued per minute
- Job execution time
- Success/failure rates
- Queue depth

### Alerts

- High failure rate
- Long-running jobs
- Queue backup

