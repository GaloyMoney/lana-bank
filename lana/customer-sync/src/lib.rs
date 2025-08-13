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
use governance::GovernanceEvent;
use keycloak_client::KeycloakConnectionConfig;
use outbox::{Outbox, OutboxEventMarker};

pub struct CustomerSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    _phantom: std::marker::PhantomData<(Perms, E)>,
    _outbox: Outbox<E>,
}

impl<Perms, E> Clone for CustomerSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
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
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn init(
        jobs: &::job::Jobs,
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        deposit: &CoreDeposit<Perms, E>,
        keycloak_connection: KeycloakConnectionConfig,
        config: CustomerSyncConfig,
    ) -> Result<Self, CustomerSyncError> {
        let keycloak_client = keycloak_client::KeycloakClient::new(
            keycloak_connection,
            config.keycloak_realm.clone(),
        );

        jobs.add_initializer_and_spawn_unique(
            CreateDepositAccountInit::new(outbox, deposit, config.clone()),
            CreateDepositAccountJobConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CreateKeycloakUserInit::new(outbox, customers, keycloak_client.clone()),
            CreateKeycloakUserJobConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            SyncEmailInit::new(outbox, customers, keycloak_client),
            SyncEmailJobConfig::new(),
        )
        .await?;
        jobs.add_initializer_and_spawn_unique(
            CustomerActiveSyncInit::new(outbox, deposit, config),
            CustomerActiveSyncJobConfig::new(),
        )
        .await?;
        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
