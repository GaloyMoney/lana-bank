use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use command_job::AtomicCommandJob;
use core_customer::{
    CUSTOMER_SYNC, CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject,
};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject,
    DepositAccountHolderStatus, GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
pub struct ActivateHolderAccountCommand {
    pub customer_id: CustomerId,
}

pub const ACTIVATE_HOLDER_ACCOUNT_COMMAND: JobType =
    JobType::new("command.customer-sync.activate-holder-account");

pub struct ActivateHolderAccountCommandJob<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    deposit: CoreDeposit<Perms, E>,
}

impl<Perms, E> ActivateHolderAccountCommandJob<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(deposit: CoreDeposit<Perms, E>) -> Self {
        Self { deposit }
    }
}

#[async_trait]
impl<Perms, E> AtomicCommandJob for ActivateHolderAccountCommandJob<Perms, E>
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
    type Command = ActivateHolderAccountCommand;

    fn job_type() -> JobType {
        ACTIVATE_HOLDER_ACCOUNT_COMMAND
    }

    fn entity_id(command: &Self::Command) -> String {
        command.customer_id.to_string()
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.activate_holder_account_job.process_command",
        skip(self, op, command),
        fields(customer_id = %command.customer_id),
    )]
    async fn run(
        &self,
        op: &mut es_entity::DbOp<'static>,
        command: &Self::Command,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.deposit
            .update_account_status_for_holder_in_op(
                op,
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(CUSTOMER_SYNC),
                command.customer_id,
                DepositAccountHolderStatus::Active,
            )
            .await?;
        Ok(())
    }
}
