mod cala;
pub mod error;
mod job;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    job::{error::JobError, Jobs},
    primitives::{CustomerId, JobId, PriceOfOneBTC},
};

use cala::*;
use error::ExportError;
use job::{DataExportConfig, DataExportInitializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntityEventData {
    id: uuid::Uuid,
    event_type: String,
    event: String,
    sequence: usize,
    recorded_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SumsubContentType {
    Webhook,
    SensitiveInfo,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportSumsubApplicantData {
    pub customer_id: CustomerId,
    pub content_type: SumsubContentType,
    pub content: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportPriceData {
    pub usd_cents_per_btc: PriceOfOneBTC,
    pub uploaded_at: DateTime<Utc>,
}

const SUMSUB_EXPORT_TABLE_NAME: &str = "sumsub_applicants";
const PRICE_EXPORT_TABLE_NAME: &str = "price_cents_btc";

#[derive(Clone)]
pub struct Export {
    cala_url: String,
    jobs: Jobs,
}

impl Export {
    pub fn new(cala_url: String, jobs: &Jobs) -> Self {
        jobs.add_initializer(DataExportInitializer);
        Self {
            cala_url,
            jobs: jobs.clone(),
        }
    }

    pub async fn export_price_data(&self, data: ExportPriceData) -> Result<(), ExportError> {
        let cala = CalaClient::new(self.cala_url.clone());
        cala.export_price_data(PRICE_EXPORT_TABLE_NAME, data)
            .await?;
        Ok(())
    }

    pub async fn export_sum_sub_applicant_data(
        &self,
        data: ExportSumsubApplicantData,
    ) -> Result<(), ExportError> {
        let cala = CalaClient::new(self.cala_url.clone());
        cala.export_applicant_data(SUMSUB_EXPORT_TABLE_NAME, data)
            .await?;
        Ok(())
    }

    #[instrument(name = "lana.export.export_last", skip(self, db, events), err)]
    pub async fn es_entity_export<T>(
        &self,
        db: &mut es_entity::DbOp<'_>,
        table_name: &'static str,
        events: impl Iterator<Item = &es_entity::PersistedEvent<T>>,
    ) -> Result<(), JobError>
    where
        T: es_entity::EsEvent + 'static,
        <T as es_entity::EsEvent>::EntityId: Into<uuid::Uuid> + std::fmt::Display + Copy,
    {
        for persisted_event in events {
            let id = persisted_event.entity_id.into();
            let event =
                serde_json::to_value(&persisted_event.event).expect("Couldn't serialize event");
            let event_type = event
                .get("type")
                .expect("Event must have a type")
                .as_str()
                .expect("Type must be a string")
                .to_string();
            let event = serde_json::to_string(&event).expect("Couldn't serialize event");
            let sequence = persisted_event.sequence;
            let recorded_at = persisted_event.recorded_at;
            let data = ExportEntityEventData {
                id,
                event,
                event_type,
                sequence,
                recorded_at,
            };
            self.jobs
                .create_and_spawn_in_op(
                    db,
                    JobId::new(),
                    DataExportConfig {
                        table_name: std::borrow::Cow::Borrowed(table_name),
                        cala_url: self.cala_url.clone(),
                        data,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
