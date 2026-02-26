use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_customer::PartyId;
use keycloak_client::KeycloakClient;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomerKeycloakUserConfig {
    pub email: String,
    pub party_id: PartyId,
}

pub const CREATE_CUSTOMER_KEYCLOAK_USER_COMMAND: JobType =
    JobType::new("command.customer-sync.create-customer-keycloak-user");

pub struct CreateCustomerKeycloakUserJobInitializer {
    keycloak_client: KeycloakClient,
}

impl CreateCustomerKeycloakUserJobInitializer {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl JobInitializer for CreateCustomerKeycloakUserJobInitializer {
    type Config = CreateCustomerKeycloakUserConfig;

    fn job_type(&self) -> JobType {
        CREATE_CUSTOMER_KEYCLOAK_USER_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreateCustomerKeycloakUserJobRunner {
            config: job.config()?,
            keycloak_client: self.keycloak_client.clone(),
        }))
    }
}

pub struct CreateCustomerKeycloakUserJobRunner {
    config: CreateCustomerKeycloakUserConfig,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl JobRunner for CreateCustomerKeycloakUserJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.create_customer_keycloak_user_job.process_command",
        skip(self, _current_job),
        fields(party_id = %self.config.party_id),
    )]
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
