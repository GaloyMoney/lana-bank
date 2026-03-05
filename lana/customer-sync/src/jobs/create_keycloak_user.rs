use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_customer::PartyId;
use job::*;
use keycloak_client::KeycloakClient;
use tracing_macros::observe_error;

pub const CREATE_KEYCLOAK_USER_COMMAND: JobType =
    JobType::new("command.customer-sync.create-keycloak-user");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeycloakUserConfig {
    pub email: String,
    pub party_id: PartyId,
}

pub struct CreateKeycloakUserJobInitializer {
    keycloak_client: KeycloakClient,
}

impl CreateKeycloakUserJobInitializer {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl JobInitializer for CreateKeycloakUserJobInitializer {
    type Config = CreateKeycloakUserConfig;

    fn job_type(&self) -> JobType {
        CREATE_KEYCLOAK_USER_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreateKeycloakUserJobRunner {
            config: job.config()?,
            keycloak_client: self.keycloak_client.clone(),
        }))
    }
}

pub struct CreateKeycloakUserJobRunner {
    config: CreateKeycloakUserConfig,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl JobRunner for CreateKeycloakUserJobRunner {
    #[observe_error(allow_single_error_alert)]
    #[tracing::instrument(name = "customer_sync.create_keycloak_user.process_command", skip_all)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.keycloak_client
            .create_user(self.config.email.clone(), self.config.party_id.into())
            .await?;
        Ok(JobCompletion::Complete)
    }
}
