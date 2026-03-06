use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
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
pub struct FreezeCustomerDepositsConfig {
    pub customer_id: CustomerId,
    pub party_id: core_customer::PartyId,
}

pub const FREEZE_CUSTOMER_DEPOSITS_COMMAND: JobType =
    JobType::new("command.customer-sync.freeze-customer-deposits");

pub struct FreezeCustomerDepositsJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    deposit: CoreDeposit<Perms, E>,
    keycloak_client: keycloak_client::KeycloakClient,
}

impl<Perms, E> FreezeCustomerDepositsJobInitializer<Perms, E>
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

impl<Perms, E> JobInitializer for FreezeCustomerDepositsJobInitializer<Perms, E>
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
    type Config = FreezeCustomerDepositsConfig;

    fn job_type(&self) -> JobType {
        FREEZE_CUSTOMER_DEPOSITS_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(FreezeCustomerDepositsJobRunner {
            config: job.config()?,
            deposit: self.deposit.clone(),
            keycloak_client: self.keycloak_client.clone(),
        }))
    }
}

pub struct FreezeCustomerDepositsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    config: FreezeCustomerDepositsConfig,
    deposit: CoreDeposit<Perms, E>,
    keycloak_client: keycloak_client::KeycloakClient,
}

#[async_trait]
impl<Perms, E> JobRunner for FreezeCustomerDepositsJobRunner<Perms, E>
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
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.freeze_customer_deposits_job.process_command",
        skip(self, current_job),
        fields(customer_id = %self.config.customer_id),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        self.deposit
            .freeze_accounts_for_holder_in_op(
                &mut op,
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(CUSTOMER_SYNC),
                self.config.customer_id,
            )
            .await?;

        if let Err(e) = self
            .keycloak_client
            .disable_user(self.config.party_id.into())
            .await
        {
            tracing::warn!("Failed to disable Keycloak user: {e}");
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
