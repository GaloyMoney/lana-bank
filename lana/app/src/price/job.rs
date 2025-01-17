use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::Duration;

use crate::{job::*, price::Price};

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct ExportPriceJobConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_export_price_interval")]
    pub job_interval_secs: Duration,
}
impl JobConfig for ExportPriceJobConfig {
    type Initializer = ExportPriceInitializer;
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

pub struct ExportPriceInitializer {
    price: Price,
}

impl ExportPriceInitializer {
    pub fn new(price: &Price) -> Self {
        Self {
            price: price.clone(),
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
            config: job.config()?,
            price: self.price.clone(),
        }))
    }
}

pub struct ExportPriceJobRunner {
    config: ExportPriceJobConfig,
    price: Price,
}

#[async_trait]
impl JobRunner for ExportPriceJobRunner {
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let _price = self.price.usd_cents_per_btc().await?;
        // self.export
        //     .export_price_data(ExportPriceData {
        //         usd_cents_per_btc: price,
        //         uploaded_at: Utc::now(),
        //     })
        //     .await?;

        Ok(JobCompletion::RescheduleIn(self.config.job_interval_secs))
    }
}
