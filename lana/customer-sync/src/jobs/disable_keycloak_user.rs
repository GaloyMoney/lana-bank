use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_customer::PartyId;
use job::*;
use keycloak_client::KeycloakClient;
use tracing_macros::record_error_severity;

pub const DISABLE_KEYCLOAK_USER_COMMAND: JobType =
    JobType::new("command.customer-sync.disable-keycloak-user");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DisableKeycloakUserConfig {
    pub party_id: PartyId,
}

pub struct DisableKeycloakUserJobInitializer {
    keycloak_client: KeycloakClient,
}

impl DisableKeycloakUserJobInitializer {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl JobInitializer for DisableKeycloakUserJobInitializer {
    type Config = DisableKeycloakUserConfig;

    fn job_type(&self) -> JobType {
        DISABLE_KEYCLOAK_USER_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DisableKeycloakUserJobRunner {
            config: job.config()?,
            keycloak_client: self.keycloak_client.clone(),
        }))
    }
}

pub struct DisableKeycloakUserJobRunner {
    config: DisableKeycloakUserConfig,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl JobRunner for DisableKeycloakUserJobRunner {
    #[record_error_severity]
    #[tracing::instrument(name = "customer_sync.disable_keycloak_user.process_command", skip_all)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.keycloak_client
            .disable_user(self.config.party_id.into())
            .await?;
        Ok(JobCompletion::Complete)
    }
}
