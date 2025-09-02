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
use es_entity::prelude::sqlx;
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use outbox::{Outbox, OutboxEventMarker};

pub use config::CustomerActivityCheckConfig;
pub use core_customer::CustomerActivityRepo;
pub use jobs::{
    CustomerActivityCheckInit, CustomerActivityCheckJobConfig, CustomerActivityUpdateConfig,
    CustomerActivityUpdateInit,
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
        pool: sqlx::PgPool,
        customers: &Customers<Perms, E>,
        deposit: &CoreDeposit<Perms, E>,
        activity_check_config: CustomerActivityCheckConfig,
    ) -> Result<Self, CustomerActivityError> {
        jobs.add_initializer_and_spawn_unique(
            CustomerActivityUpdateInit::new(outbox, pool.clone(), deposit),
            CustomerActivityUpdateConfig::new(),
        )
        .await?;

        jobs.add_initializer_and_spawn_unique(
            CustomerActivityCheckInit::new(customers, pool, activity_check_config),
            CustomerActivityCheckJobConfig::new(),
        )
        .await?;

        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}
