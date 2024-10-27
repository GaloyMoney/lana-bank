use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::Duration;

use crate::{
    data_export::{Export, ExportPriceData},
    job::*,
    price::Price,
};

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct ExportPriceJobConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_export_price_interval")]
    pub job_interval_secs: Duration,
}

fn default_export_price_interval() -> Duration {
    Duration::from_secs(60)
}

impl Default for ExportPriceJobConfig {
    fn default() -> Self {
        Self {
            job_interval_secs: default_export_price_interval(),
        }
    }
}

impl ExportPriceJobConfig {
    fn next_run_at(&self) -> DateTime<Utc> {
        Utc::now() + self.job_interval_secs
    }
}

pub struct ExportPriceInitializer {
    price: Price,
    export: Export,
}

impl ExportPriceInitializer {
    pub fn new(price: &Price, export: &Export) -> Self {
        Self {
            price: price.clone(),
            export: export.clone(),
        }
    }
}

const PRICE_EXPORT_JOB: JobType = JobType::new("price-export");
impl JobInitializer for ExportPriceInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        PRICE_EXPORT_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ExportPriceJobRunner {
            config: job.data()?,
            price: self.price.clone(),
            export: self.export.clone(),
        }))
    }
}

pub struct ExportPriceJobRunner {
    config: ExportPriceJobConfig,
    price: Price,
    export: Export,
}

#[async_trait]
impl JobRunner for ExportPriceJobRunner {
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = self.price.usd_cents_per_btc().await?;
        self.export
            .export_price_data(ExportPriceData {
                usd_cents_per_btc: price,
                uploaded_at: Utc::now(),
            })
            .await?;

        Ok(JobCompletion::RescheduleAt(self.config.next_run_at()))
    }
}
