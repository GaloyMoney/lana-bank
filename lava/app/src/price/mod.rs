mod bfx_client;
pub mod error;
use cached::proc_macro::cached;
mod job;

use crate::{
    constants::PRICE_JOB_ID,
    data_export::Export,
    job::Jobs,
    primitives::{PriceOfOneBTC, UsdCents},
};

use bfx_client::BfxClient;
use error::PriceError;

#[derive(Clone)]
pub struct Price {
    bfx: BfxClient,
    pool: sqlx::PgPool,
    jobs: Jobs,
}

impl Price {
    pub fn new(pool: &sqlx::PgPool, jobs: &Jobs, export: &Export) -> Self {
        let price = Self {
            bfx: BfxClient::new(),
            pool: pool.clone(),
            jobs: jobs.clone(),
        };

        jobs.add_initializer(job::ExportPriceInitializer::new(&price, export));
        price
    }

    pub async fn usd_cents_per_btc(&self) -> Result<PriceOfOneBTC, PriceError> {
        usd_cents_per_btc_cached(&self.bfx).await
    }

    pub async fn spawn_global_jobs(&self) -> Result<(), PriceError> {
        let mut db_tx = self.pool.begin().await?;
        match self
            .jobs
            .create_and_spawn_in_tx::<job::ExportPriceInitializer, _>(
                &mut db_tx,
                PRICE_JOB_ID,
                "export-price-job".to_string(),
                job::ExportPriceJobConfig::default(),
            )
            .await
        {
            Err(crate::job::error::JobError::DuplicateId) => (),
            Err(e) => return Err(e.into()),
            _ => (),
        }
        db_tx.commit().await?;
        Ok(())
    }
}

#[cached(time = 60, result = true, key = "()", convert = r#"{}"#)]
async fn usd_cents_per_btc_cached(bfx: &BfxClient) -> Result<PriceOfOneBTC, PriceError> {
    let last_price = bfx.btc_usd_tick().await?.last_price;
    Ok(PriceOfOneBTC::new(UsdCents::try_from_usd(last_price)?))
}
