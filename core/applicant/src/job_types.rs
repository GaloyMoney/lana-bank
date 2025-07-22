use serde::{Deserialize, Serialize};

use job::JobType;

/// Job configuration for Sumsub transaction export  
/// Note: JobConfig implementation should be provided by the application layer
#[derive(Clone, Serialize, Deserialize)]
pub struct SumsubExportJobConfig;

/// Job type constant for Sumsub export
pub const SUMSUB_EXPORT_JOB: JobType = JobType::new("sumsub-export");

/// Job execution state for tracking progress
#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct SumsubExportJobData {
    pub sequence: outbox::EventSequence,
}
