pub mod cala;
mod job;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use tracing::instrument;

use crate::{
    entity::{EntityEvent, EntityEvents},
    job::{error::JobError, Jobs},
    primitives::{CustomerId, JobId},
};

use cala::{error::CalaError, *};
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
    Fetched,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportSumsubApplicantData {
    pub customer_id: CustomerId,
    pub content_type: SumsubContentType,
    pub content: String,
    pub uploaded_at: DateTime<Utc>,
}

const SUMSUB_EXPORT_TABLE_NAME: &str = "sumsub_applicants";

#[derive(Clone)]
pub struct Export {
    cala_url: String,
    jobs: Jobs,
}

impl Export {
    pub fn new(cala_url: String, jobs: &Jobs) -> Self {
        jobs.add_initializer(DataExportInitializer::new());
        Self {
            cala_url,
            jobs: jobs.clone(),
        }
    }

    pub async fn export_sum_sub_applicant_data(
        &self,
        data: ExportSumsubApplicantData,
    ) -> Result<(), CalaError> {
        let cala = CalaClient::new(self.cala_url.clone());
        cala.export_applicant_data(SUMSUB_EXPORT_TABLE_NAME, data)
            .await?;
        Ok(())
    }

    #[instrument(name = "lava.export.export_last", skip(self, db, events), err)]
    pub async fn export_last<T: EntityEvent + 'static>(
        &self,
        db: &mut Transaction<'_, Postgres>,
        table_name: &'static str,
        last: usize,
        events: &EntityEvents<T>,
    ) -> Result<(), JobError> {
        let id: uuid::Uuid = events.entity_id.into();
        let recorded_at = events
            .latest_event_persisted_at
            .expect("No events persisted");
        for (sequence, event) in events.last_persisted(last) {
            let event = serde_json::to_value(event).expect("Couldn't serialize event");
            let event_type = event
                .get("type")
                .expect("Event must have a type")
                .as_str()
                .expect("Type must be a string")
                .to_string();
            let event = serde_json::to_string(&event).expect("Couldn't serialize event");
            let data = ExportEntityEventData {
                id,
                event,
                event_type,
                sequence,
                recorded_at,
            };
            self.jobs
                .create_and_spawn_job::<DataExportInitializer, _>(
                    db,
                    JobId::new(),
                    format!("export:{}:{}", id, sequence),
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
