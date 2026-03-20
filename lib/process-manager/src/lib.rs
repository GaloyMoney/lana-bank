//! Process-manager utility helpers
//!
//! Standalone async functions that reduce boilerplate in process-manager jobs.
//! These are **complementary** to the job framework — each function takes
//! `&mut CurrentJob` (or an `&mut DbOp`) and handles one recurring pattern:
//!
//! - [`capture_and_spawn`] — capture outbox sequence + spawn + persist state
//! - [`await_event`] — single-event outbox stream with shutdown handling
//! - [`await_events`] — fan-in outbox stream over a pending set
//! - [`await_job_completions`] — join-all with shutdown handling

use std::collections::HashSet;
use std::hash::Hash;

use futures::StreamExt;
use serde::{Serialize, de::DeserializeOwned};

use job::*;
use obix::{
    EventSequence,
    out::{Outbox, OutboxEventMarker},
};

/// Query the highest known persistent outbox sequence from the database.
///
/// This is useful for recording the current outbox position before spawning
/// child jobs, so we can later listen from that position and not miss events.
pub async fn current_outbox_sequence(
    pool: &sqlx::PgPool,
) -> Result<EventSequence, Box<dyn std::error::Error>> {
    let max_seq: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(sequence), 0) FROM cala_persistent_outbox_events")
            .fetch_one(pool)
            .await?;
    Ok(EventSequence::from(max_seq as u64))
}

/// Capture the current outbox sequence, spawn child jobs inside an op,
/// persist a state update, and return `RescheduleNowWithOp`.
///
/// Used by event-driven PMs that need to atomically record the sequence
/// number they will listen from alongside the spawn.
///
/// Call [`current_outbox_sequence`] beforehand to obtain the sequence,
/// then pass it here.
///
/// `state_update_fn` receives the outbox sequence number and should return
/// the new execution state to persist.
pub async fn capture_and_spawn<C, S>(
    current_job: &mut CurrentJob,
    start_sequence: EventSequence,
    spawner: &JobSpawner<C>,
    specs: Vec<JobSpec<C>>,
    state_update_fn: impl FnOnce(EventSequence) -> S,
) -> Result<JobCompletion, Box<dyn std::error::Error>>
where
    C: Serialize + DeserializeOwned + Send + Sync,
    S: Serialize + Send + Sync,
{
    let mut op = current_job.begin_op().await?;

    spawner.spawn_all_in_op(&mut op, specs).await?;

    let new_state = state_update_fn(start_sequence);
    current_job
        .update_execution_state_in_op(&mut op, &new_state)
        .await?;
    Ok(JobCompletion::RescheduleNowWithOp(op))
}

/// Stream outbox events, waiting for a single event that matches `filter_fn`.
///
/// - Returns `Some(sequence)` when `filter_fn` returns `true` for an event.
/// - Returns `None` on shutdown (caller should reschedule).
///
/// `filter_fn` receives the deserialized domain event and returns `true` if
/// the event is the one we are waiting for.
pub async fn await_event<E, T>(
    current_job: &mut CurrentJob,
    outbox: &Outbox<E>,
    start_sequence: EventSequence,
    filter_fn: impl Fn(&T) -> bool,
) -> Result<Option<EventSequence>, Box<dyn std::error::Error>>
where
    E: OutboxEventMarker<T>,
    T: Send + Sync,
{
    let mut stream = outbox.listen_persisted(start_sequence);

    loop {
        tokio::select! {
            Some(event) = stream.next() => {
                let matched = event.payload.as_ref()
                    .and_then(|p| p.as_event())
                    .map(&filter_fn)
                    .unwrap_or(false);

                if matched {
                    return Ok(Some(event.sequence));
                }
            }
            _ = current_job.shutdown_requested() => {
                return Ok(None);
            }
        }
    }
}

/// Fan-in outbox stream over a pending set with shutdown handling.
///
/// Keeps consuming events until all items in `pending` are matched.
/// `filter_fn` receives the deserialized domain event and returns
/// `Some(key)` to identify which pending item was completed.
///
/// Returns:
/// - `Ok(Some(()))` — all pending items matched
/// - `Ok(None)` — shutdown requested (caller should persist and reschedule)
pub async fn await_events<E, T, K>(
    current_job: &mut CurrentJob,
    outbox: &Outbox<E>,
    pending: &mut HashSet<K>,
    start_sequence: &mut EventSequence,
    filter_fn: impl Fn(&T) -> Option<K>,
) -> Result<Option<()>, Box<dyn std::error::Error>>
where
    E: OutboxEventMarker<T>,
    T: Send + Sync,
    K: Eq + Hash,
{
    if pending.is_empty() {
        return Ok(Some(()));
    }

    let mut stream = outbox.listen_persisted(*start_sequence);

    loop {
        tokio::select! {
            Some(event) = stream.next() => {
                let matched_key = event.payload.as_ref()
                    .and_then(|p| p.as_event())
                    .and_then(&filter_fn);

                if let Some(key) = matched_key
                    && pending.remove(&key)
                {
                    *start_sequence = event.sequence;
                }
                if pending.is_empty() {
                    return Ok(Some(()));
                }
            }
            _ = current_job.shutdown_requested() => {
                return Ok(None);
            }
        }
    }
}

/// Await completion of all given job IDs with shutdown handling.
///
/// Uses `futures::future::join_all` + `tokio::select!` for graceful shutdown.
///
/// Returns:
/// - `Ok(Some(results))` — all jobs finished, with their terminal states
/// - `Ok(None)` — shutdown requested (caller should reschedule)
pub async fn await_job_completions(
    current_job: &mut CurrentJob,
    jobs: &Jobs,
    job_ids: &[JobId],
) -> Result<Option<Vec<job::JobCompletionResult>>, Box<dyn std::error::Error>> {
    if job_ids.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let futures: Vec<_> = job_ids
        .iter()
        .map(|job_id| jobs.await_completion(*job_id, None))
        .collect();

    let results = tokio::select! {
        results = futures::future::join_all(futures) => results,
        _ = current_job.shutdown_requested() => {
            return Ok(None);
        }
    };

    let mut terminals = Vec::with_capacity(results.len());
    for result in results {
        terminals.push(result?);
    }
    Ok(Some(terminals))
}

/// Check whether all terminal states are `Completed`.
/// Convenience for callers that just want a pass/fail summary.
pub fn all_completed(results: &[job::JobCompletionResult]) -> bool {
    results
        .iter()
        .all(|r| r.state() == job::JobTerminalState::Completed)
}

/// Count how many terminal states are not `Completed`.
pub fn failed_count(results: &[job::JobCompletionResult]) -> usize {
    results
        .iter()
        .filter(|r| r.state() != job::JobTerminalState::Completed)
        .count()
}
