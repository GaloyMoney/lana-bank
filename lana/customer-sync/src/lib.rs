#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod error;
mod job;

use config::*;
use error::*;
use job::*;

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

pub struct CustomerSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    _phantom: std::marker::PhantomData<(Perms, E)>,
    _outbox: Outbox<E>,
}

impl<Perms, E> Clone for CustomerSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> CustomerSync<Perms, E>
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
        config: CustomerSyncConfig,
    ) -> Result<Self, CustomerSyncError> {
        let keycloak_client = keycloak_client::KeycloakClient::new(config.keycloak.clone());

        jobs.add_initializer_and_spawn_unique(
            CreateDepositAccountInit::new(outbox, deposit, config.clone()),
            CreateDepositAccountJobConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CreateKeycloakUserInit::new(outbox, keycloak_client.clone()),
            CreateKeycloakUserJobConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            SyncEmailInit::new(outbox, keycloak_client),
            SyncEmailJobConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CustomerActiveSyncInit::new(outbox, deposit, config.clone()),
            CustomerActiveSyncJobConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            customer_activity::CustomerActivityProjectionInit::new(outbox, pool.clone(), deposit),
            customer_activity::CustomerActivityProjectionConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CustomerActivityCheckInit::new(customers, pool, config),
            CustomerActivityCheckJobConfig::new(),
        )
        .await?;
        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
