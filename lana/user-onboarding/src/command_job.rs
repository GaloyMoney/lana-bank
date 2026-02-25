use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_access::UserId;
use keycloak_client::KeycloakClient;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateKeycloakUserConfig {
    pub email: String,
    pub user_id: UserId,
}

pub const CREATE_KEYCLOAK_USER_COMMAND: JobType =
    JobType::new("command.user-onboarding.create-keycloak-user");

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
    // TODO: restore #[record_error_severity] after removing test hack
    #[tracing::instrument(
        name = "user_onboarding.create_keycloak_user_job.process_command",
        skip(self, _current_job),
        fields(user_id = %self.config.user_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.keycloak_client
            .create_user(self.config.email.clone(), self.config.user_id.into())
            .await?;
        // TODO: remove this - temporary error to test Honeycomb error panel
        Err("TEMPORARY: testing command job error panel".into())
    }
}
