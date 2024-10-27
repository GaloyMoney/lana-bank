#![allow(clippy::blocks_in_conditions)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

use crate::job::*;

use super::{cala::CalaClient, ExportEntityEventData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportConfig {
    pub(super) cala_url: String,
    pub(super) table_name: Cow<'static, str>,
    pub(super) data: ExportEntityEventData,
}

pub struct DataExportInitializer {}

impl DataExportInitializer {
    pub fn new() -> Self {
        Self {}
    }
}

const DATA_EXPORT_JOB: JobType = JobType::new("data-export");
impl JobInitializer for DataExportInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DATA_EXPORT_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DataExportJobRunner {
            config: job.data()?,
        }))
    }
}

pub struct DataExportJobRunner {
    config: DataExportConfig,
}

#[async_trait]
impl JobRunner for DataExportJobRunner {
    #[tracing::instrument(name = "lava.data_export.job.run", skip_all, fields(insert_id), err)]
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let cala = CalaClient::new(self.config.cala_url.clone());
        cala.export_entity_event_to_bq(&self.config.table_name, &self.config.data)
            .await?;
        Ok(JobCompletion::Complete)
    }
}
