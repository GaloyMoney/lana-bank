use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use keycloak_client::KeycloakClient;
use tracing_macros::record_error_severity;

use core_customer::PartyId;

#[derive(Serialize, Deserialize, Clone)]
pub struct SyncEmailConfig {
    pub customer_id: PartyId,
    pub email: String,
}

pub const SYNC_EMAIL_TASK: JobType = JobType::new("task.customer-sync.sync-email");

pub struct SyncEmailJobInitializer {
    keycloak_client: KeycloakClient,
}

impl SyncEmailJobInitializer {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl JobInitializer for SyncEmailJobInitializer {
    type Config = SyncEmailConfig;

    fn job_type(&self) -> JobType {
        SYNC_EMAIL_TASK
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SyncEmailJobRunner {
            config: job.config()?,
            keycloak_client: self.keycloak_client.clone(),
        }))
    }
}

pub struct SyncEmailJobRunner {
    config: SyncEmailConfig,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl JobRunner for SyncEmailJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.sync_email_job.run",
        skip(self, _current_job),
        fields(customer_id = %self.config.customer_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.keycloak_client
            .update_user_email(self.config.customer_id.into(), self.config.email.clone())
            .await?;
        Ok(JobCompletion::Complete)
    }
}
