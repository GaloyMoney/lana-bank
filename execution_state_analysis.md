# Execution State Analysis for Job Library

## Overview

The `execution_state` field in the job library is designed to store intermediate state data for long-running jobs that need to persist progress between execution attempts or interruptions.

## Field Definition

```rust
execution_state: Option<serde_json::Value>,
```

**Location**: `lib/job/src/current.rs` line 9

## Database Schema

The field is stored in the `job_executions` table as a JSONB column:
```sql
execution_state_json JSONB,
```

**Location**: `lana/app/migrations/20240517074612_core_setup.sql` line 432

## Usage Patterns

### 1. **Event Stream Processing Jobs**
Most jobs that use execution state are event-driven jobs that process streams of events:

**Pattern**: Store the last processed event sequence number to avoid reprocessing events on restart.

**Examples**:
- `CreditFacilityApprovalJobData`: Stores `outbox::EventSequence`
- `DashboardProjectionJobData`: Stores `outbox::EventSequence` + `DashboardValues` 
- `CreditFacilityCollateralizationFromEventsData`: Stores `outbox::EventSequence`
- `EmailEventListenerJobData`: Stores `outbox::EventSequence`

**Code Pattern**:
```rust
#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct JobData {
    sequence: outbox::EventSequence,
}

// Usage in job runner
let mut state = current_job
    .execution_state::<JobData>()?
    .unwrap_or_default();
    
// Process events...
state.sequence = message.sequence;
current_job.update_execution_state(state).await?;
```

### 2. **Multi-Step Process Jobs**
Jobs that have multiple steps and need to track progress:

**Examples**:
- `WithdrawApprovalJobData`: Process approval workflows
- `DisbursalApprovalJobData`: Process disbursement approvals
- `CreateDepositAccountJobData`: Multi-step account creation

### 3. **Projection/Aggregation Jobs**
Jobs that maintain derived state from events:

**Example**: `DashboardProjectionJobData` stores both sequence and aggregated dashboard values:
```rust
#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct DashboardProjectionJobData {
    sequence: outbox::EventSequence,
    dashboard: DashboardValues,  // Aggregated state
}
```

## API Usage

### Reading Execution State
```rust
let state = current_job
    .execution_state::<YourJobData>()?
    .unwrap_or_default();
```

### Updating Execution State
```rust
// Simple update
current_job.update_execution_state(state).await?;

// Update within transaction
current_job
    .update_execution_state_in_tx(&mut db, &state)
    .await?;
```

## Why It's Untyped (serde_json::Value)

The execution state is stored as `serde_json::Value` for several architectural reasons:

### 1. **Job Type Polymorphism**
The job system is designed to handle multiple job types through the same infrastructure. Each job type has its own specific execution state structure:

- `CreditFacilityApprovalJobData`
- `DashboardProjectionJobData` 
- `UserOnboardingJobData`
- etc.

Using `serde_json::Value` allows the job infrastructure to store and retrieve any serializable type without needing to know the specific type at compile time.

### 2. **Generic API Design**
The `CurrentJob` struct provides a generic API:
```rust
pub fn execution_state<T: DeserializeOwned>(&self) -> Result<Option<T>, serde_json::Error>
```

This allows job implementations to specify their own types while the infrastructure remains generic.

### 3. **Database Storage Flexibility**
Storing as JSONB in PostgreSQL provides:
- Flexible schema evolution
- Efficient storage for nested data
- Query capabilities when needed
- No need for separate tables per job type

### 4. **Serialization at Boundaries**
The type-safe serialization/deserialization happens at the job implementation boundaries:
- Jobs serialize their typed state to JSON when updating
- Jobs deserialize from JSON to their typed state when reading
- The infrastructure only handles the JSON transport

## Error Handling

There's a specific error type for serialization failures:
```rust
#[error("JobError - BadState: {0}")]
CouldNotSerializeExecutionState(serde_json::Error),
```

This ensures that serialization errors are properly categorized and handled.

## Design Benefits

1. **Type Safety at Job Level**: Each job gets compile-time type safety for its own state
2. **Infrastructure Simplicity**: Job framework doesn't need to be generic over state types
3. **Storage Efficiency**: Single table design with flexible JSONB storage
4. **Backwards Compatibility**: Easy to evolve job state structures using Serde's defaults
5. **Database Agnostic**: JSON is universally supported across databases

## Common Usage Pattern Summary

The most common pattern is event stream processing where jobs need to:
1. Track the last processed event sequence
2. Resume from that point on restart/retry
3. Update the sequence after successful processing
4. Sometimes maintain additional derived state alongside the sequence

This design allows the job system to provide exactly-once processing guarantees and resilient job execution across failures and restarts.