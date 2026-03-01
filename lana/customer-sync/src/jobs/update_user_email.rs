use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_customer::PartyId;
use job::*;
use keycloak_client::KeycloakClient;
use tracing_macros::record_error_severity;

pub const UPDATE_USER_EMAIL_COMMAND: JobType =
    JobType::new("command.customer-sync.update-user-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserEmailConfig {
    pub party_id: PartyId,
    pub email: String,
}

pub struct UpdateUserEmailJobInitializer {
    keycloak_client: KeycloakClient,
}

impl UpdateUserEmailJobInitializer {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

impl JobInitializer for UpdateUserEmailJobInitializer {
    type Config = UpdateUserEmailConfig;

    fn job_type(&self) -> JobType {
        UPDATE_USER_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateUserEmailJobRunner {
            config: job.config()?,
            keycloak_client: self.keycloak_client.clone(),
        }))
    }
}

pub struct UpdateUserEmailJobRunner {
    config: UpdateUserEmailConfig,
    keycloak_client: KeycloakClient,
}

#[async_trait]
impl JobRunner for UpdateUserEmailJobRunner {
    #[record_error_severity]
    #[tracing::instrument(name = "customer_sync.update_user_email.process_command", skip_all)]
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
