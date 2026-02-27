use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_customer::PartyId;
use keycloak_client::KeycloakClient;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncKeycloakEmailConfig {
    pub party_id: PartyId,
    pub email: String,
}

pub const SYNC_KEYCLOAK_EMAIL_COMMAND: JobType =
    JobType::new("command.customer-sync.sync-keycloak-email");

pub struct SyncKeycloakEmailJobInitializer {
    keycloak_client: KeycloakClient,
}

impl SyncKeycloakEmailJobInitializer {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl JobInitializer for SyncKeycloakEmailJobInitializer {
    type Config = SyncKeycloakEmailConfig;

    fn job_type(&self) -> JobType {
        SYNC_KEYCLOAK_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SyncKeycloakEmailJobRunner {
            config: job.config()?,
            keycloak_client: self.keycloak_client.clone(),
        }))
    }
}

pub struct SyncKeycloakEmailJobRunner {
    config: SyncKeycloakEmailConfig,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl JobRunner for SyncKeycloakEmailJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.sync_keycloak_email_job.process_command",
        skip(self, _current_job),
        fields(party_id = %self.config.party_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.keycloak_client
            .update_user_email(self.config.party_id.into(), self.config.email.clone())
            .await?;
        Ok(JobCompletion::Complete)
    }
}
