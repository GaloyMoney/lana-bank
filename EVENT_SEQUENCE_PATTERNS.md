# Event Sequence Update Patterns in Job System

## Overview

This document explains two different patterns for updating `EventSequence` in permanent event-processing jobs, their tradeoffs, and provides guidance for choosing between them.

## Background

Permanent jobs in our system process event streams from the outbox pattern. They store an `EventSequence` in their execution state to track which events they've processed, enabling exactly-once processing guarantees across job restarts.

```rust
#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct JobData {
    sequence: outbox::EventSequence,
}
```

The critical question is: **When should we update the sequence?**

## The Two Patterns

### Pattern A: Update Every Event (Universal Updates)

**Philosophy**: Move forward through ALL events, regardless of relevance.

```rust
async fn run(&self, mut current_job: CurrentJob) -> Result<JobCompletion, _> {
    let mut state = current_job.execution_state::<JobData>()?.unwrap_or_default();
    let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

    while let Some(message) = stream.next().await {
        // Process only relevant events
        if let Some(RelevantEvent { .. }) = message.as_ref().as_event() {
            process_event().await?;
        }
        
        // ✅ ALWAYS update sequence for every event
        state.sequence = message.sequence;
        current_job.update_execution_state(state).await?;
    }
    
    Ok(JobCompletion::RescheduleNow)
}
```

### Pattern B: Update Only Processed Events (Selective Updates)

**Philosophy**: Only advance sequence when we actually process something.

```rust
async fn run(&self, mut current_job: CurrentJob) -> Result<JobCompletion, _> {
    let mut state = current_job.execution_state::<JobData>()?.unwrap_or_default();
    let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

    while let Some(message) = stream.next().await {
        match message.as_ref().as_event() {
            Some(RelevantEvent { .. }) => {
                process_event().await?;
                
                // ✅ ONLY update sequence when we process something
                state.sequence = message.sequence;
                current_job.update_execution_state(state).await?;
            }
            _ => {} // Skip irrelevant events without updating sequence
        }
    }
    
    Ok(JobCompletion::RescheduleNow)
}
```

## Current Usage in Codebase

### Pattern A Jobs (4 total)
- `HistoryProjectionJobRunner`
- `RepaymentPlanProjectionJobRunner` 
- `EmailEventListenerRunner`
- `SumsubExportJobRunner`

### Pattern B Jobs (9 total)
- **Approval Jobs**: `CreditFacilityApprovalJobRunner`, `DisbursalApprovalJobRunner`, etc.
- **Customer Sync**: `SyncEmailJobRunner`, `CustomerActiveSyncJobRunner`, etc.
- **Credit Processing**: `CreditFacilityCollateralizationFromEventsData`

## Tradeoffs Analysis

### Pattern A: Universal Updates

#### ✅ Advantages
- **Guaranteed Progress**: Never gets stuck on irrelevant events
- **Simpler Logic**: No conditional sequence management
- **Resilient**: Handles changes in event relevance gracefully
- **Predictable**: Always moves forward through the stream
- **Easier Debugging**: Sequence always reflects last seen event

#### ❌ Disadvantages  
- **Extra Writes**: Updates execution state for every event
- **Database Load**: More frequent state persistence
- **Less Precise**: Sequence doesn't reflect actual work done

#### 💡 Best For
- High-volume event streams with mixed relevance
- Jobs that might change their filtering logic over time
- Critical jobs where getting stuck is unacceptable

### Pattern B: Selective Updates

#### ✅ Advantages
- **Efficient**: Only updates state when necessary
- **Precise**: Sequence reflects actual work completed
- **Performance**: Fewer database writes
- **Resource Conscious**: Minimizes unnecessary persistence

#### ❌ Disadvantages
- **Stagnation Risk**: Can get stuck on irrelevant events
- **Complex Logic**: Requires careful handling of update conditions
- **Fragile**: Changes to event filtering can cause issues
- **Debugging Harder**: Sequence might lag behind stream position

#### 💡 Best For
- Low-volume, highly filtered event streams
- Performance-critical jobs with predictable event patterns
- Jobs with stable, well-defined event filtering

## Stagnation Risk Example

Consider a job using Pattern B that only processes `UserCreated` events:

```rust
// Event Stream: [OrderCreated, ProductUpdated, OrderCreated, UserCreated]
// Pattern B Job: Only advances on UserCreated
// Sequence: 0 -> 0 -> 0 -> 4

// If stream becomes: [OrderCreated, ProductUpdated, OrderCreated, ...]
// Job gets stuck at sequence 0, reprocessing same events forever!
```

With Pattern A, the job would advance through all events, avoiding this issue.

## Recommendations

### Default Choice: Pattern A
**Recommend Pattern A for most new jobs** because:
- Safety is more important than minor performance gains
- Event streams are unpredictable in production
- The overhead is usually negligible
- Debugging is significantly easier

### When to Use Pattern B
Only use Pattern B when:
- ✅ Event stream is predictable and stable
- ✅ Performance is critical (high-volume processing)
- ✅ Event filtering logic is unlikely to change
- ✅ You understand the stagnation risks

### Migration Strategy
For existing Pattern B jobs showing stagnation issues:
1. Monitor job sequence progress vs. stream position
2. Identify jobs that aren't advancing
3. Migrate to Pattern A for problematic jobs

## Implementation Guidelines

### Pattern A Template
```rust
while let Some(message) = stream.next().await {
    // Process relevant events conditionally
    if let Some(event) = message.as_ref().as_event() {
        match event {
            RelevantEvent { .. } => process_event().await?,
            _ => {} // Ignore irrelevant events
        }
    }
    
    // Always advance sequence
    state.sequence = message.sequence;
    current_job.update_execution_state(state).await?;
}
```

### Pattern B Template
```rust
while let Some(message) = stream.next().await {
    match message.as_ref().as_event() {
        Some(RelevantEvent { .. }) => {
            process_event().await?;
            // Only advance on processed events
            state.sequence = message.sequence;
            current_job.update_execution_state(state).await?;
        }
        _ => {} // Skip without advancing
    }
}
```

## Common Bugs to Avoid

### ❌ Critical Bug: No Sequence Updates
```rust
// NEVER DO THIS - Job will reprocess all events on every restart
while let Some(message) = stream.next().await {
    if let Some(event) = message.as_ref().as_event() {
        process_event().await?;
        // Missing: state.sequence = message.sequence;
        // Missing: current_job.update_execution_state(state).await?;
    }
}
```

### ❌ State Mutability Issues
```rust
// Must be mutable for updates
let mut state = current_job.execution_state::<JobData>()?;
let mut current_job = current_job; // If updating state
```

## Monitoring

Track these metrics for event-processing jobs:
- **Sequence Lag**: Difference between job sequence and latest stream position
- **Processing Rate**: Events processed per time period  
- **Stagnation Alerts**: Jobs with sequence not advancing for X minutes

## Future Considerations

Consider enhancing the job framework with:
- Built-in stagnation detection
- Automatic pattern selection based on job characteristics
- Metrics collection for sequence advancement
- Framework-level sequence management abstractions

---

*For questions about this pattern choice, consult the architecture team or create an issue for discussion.*