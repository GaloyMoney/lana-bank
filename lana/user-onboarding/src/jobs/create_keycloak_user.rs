use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use command_job::CommandJob;
use job::*;
use keycloak_client::KeycloakClient;
use tracing_macros::record_error_severity;

use core_access::UserId;

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateKeycloakUserCommand {
    pub email: String,
    pub user_id: UserId,
}

pub const CREATE_KEYCLOAK_USER_COMMAND: JobType =
    JobType::new("command.user-onboarding.create-keycloak-user");

pub struct CreateKeycloakUserCommandJob {
    keycloak_client: KeycloakClient,
}

impl CreateKeycloakUserCommandJob {
    pub fn new(keycloak_client: KeycloakClient) -> Self {
        Self { keycloak_client }
    }
}

#[async_trait]
impl CommandJob for CreateKeycloakUserCommandJob {
    type Command = CreateKeycloakUserCommand;

    fn job_type() -> JobType {
        CREATE_KEYCLOAK_USER_COMMAND
    }

    fn entity_id(command: &Self::Command) -> String {
        command.user_id.to_string()
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "user_onboarding.create_keycloak_user_job.process_command",
        skip(self, _current_job, command),
        fields(user_id = %command.user_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
        command: &Self::Command,
    ) -> Result<JobCompletion, Box<dyn std::error::Error + Send + Sync>> {
        self.keycloak_client
            .create_user(command.email.clone(), command.user_id.into())
            .await?;
        Ok(JobCompletion::Complete)
    }
}
