use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use command_job::CommandJob;
use core_customer::{
    CUSTOMER_SYNC, CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject,
};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use governance::GovernanceEvent;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
pub struct FreezeCustomerDepositsCommand {
    pub customer_id: CustomerId,
    pub party_id: core_customer::PartyId,
}

pub const FREEZE_CUSTOMER_DEPOSITS_COMMAND: JobType =
    JobType::new("command.customer-sync.freeze-customer-deposits");

pub struct FreezeCustomerDepositsCommandJob<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    deposit: CoreDeposit<Perms, E>,
    keycloak_client: keycloak_client::KeycloakClient,
}

impl<Perms, E> FreezeCustomerDepositsCommandJob<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        deposit: CoreDeposit<Perms, E>,
        keycloak_client: keycloak_client::KeycloakClient,
    ) -> Self {
        Self {
            deposit,
            keycloak_client,
        }
    }
}

#[async_trait]
impl<Perms, E> CommandJob for FreezeCustomerDepositsCommandJob<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Command = FreezeCustomerDepositsCommand;

    fn job_type() -> JobType {
        FREEZE_CUSTOMER_DEPOSITS_COMMAND
    }

    fn queue_id(command: &Self::Command) -> String {
        command.customer_id.to_string()
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.freeze_customer_deposits_job.process_command",
        skip(self, current_job, command),
        fields(customer_id = %command.customer_id),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
        command: &Self::Command,
    ) -> Result<JobCompletion, Box<dyn std::error::Error + Send + Sync>> {
        let mut op = current_job.begin_op().await?;
        self.deposit
            .freeze_accounts_for_holder_in_op(
                &mut op,
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(CUSTOMER_SYNC),
                command.customer_id,
            )
            .await?;

        if let Err(e) = self
            .keycloak_client
            .disable_user(command.party_id.into())
            .await
        {
            tracing::warn!("Failed to disable Keycloak user: {e}");
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
