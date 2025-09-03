#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod error;
pub mod jobs;
mod time;

use error::*;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use outbox::{Outbox, OutboxEventMarker};

pub use config::UpdateCustomerActivityConfig;
pub use jobs::{
    UpdateCustomerActivityInit, UpdateCustomerActivityJobConfig, UpdateLastActivityConfig,
    UpdateLastActivityInit,
};

pub struct CustomerActivityJobs<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> Clone for CustomerActivityJobs<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> CustomerActivityJobs<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn init(
        jobs: &::job::Jobs,
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        deposit: &CoreDeposit<Perms, E>,
        update_customer_activity_config: UpdateCustomerActivityConfig,
    ) -> Result<Self, CustomerActivityError> {
        jobs.add_initializer_and_spawn_unique(
            UpdateLastActivityInit::new(outbox, &customers.clone(), deposit),
            UpdateLastActivityConfig::new(),
        )
        .await?;

        jobs.add_initializer_and_spawn_unique(
            UpdateCustomerActivityInit::new(customers, update_customer_activity_config),
            UpdateCustomerActivityJobConfig::new(),
        )
        .await?;

        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}
