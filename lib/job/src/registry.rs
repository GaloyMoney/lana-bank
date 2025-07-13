use std::collections::HashMap;

use super::{entity::*, error::JobError, traits::*};

pub struct JobRegistry {
    initializers: HashMap<JobType, Box<dyn JobInitializer>>,
    retry_settings: HashMap<JobType, RetrySettings>,
}

impl JobRegistry {
    pub(crate) fn new() -> Self {
        Self {
            initializers: HashMap::new(),
            retry_settings: HashMap::new(),
        }
    }

    pub fn add_initializer<I: JobInitializer>(&mut self, initializer: I) {
        self.initializers
            .insert(<I as JobInitializer>::job_type(), Box::new(initializer));
        self.retry_settings.insert(
            <I as JobInitializer>::job_type(),
            <I as JobInitializer>::retry_on_error_settings(),
        );
    }

    pub(super) fn init_job(&self, job: &Job) -> Result<Box<dyn JobRunner>, JobError> {
        self.initializers
            .get(&job.job_type)
            .ok_or(JobError::NoInitializerPresent)?
            .init(job)
            .map_err(|e| JobError::JobInitError(e.to_string()))
    }

    pub(super) fn retry_settings(&self, job_type: &JobType) -> &RetrySettings {
        self.retry_settings
            .get(job_type)
            .expect("Retry settings not found")
    }
}
