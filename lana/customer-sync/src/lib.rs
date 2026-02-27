#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod error;
mod jobs;

use config::*;
use error::*;
use jobs::*;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use core_time_events::CoreTimeEvent;
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use obix::out::{Outbox, OutboxEventJobConfig, OutboxEventMarker};
use tracing_macros::record_error_severity;

pub struct CustomerSync<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
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
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
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
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "customer_sync.init", skip_all)]
    pub async fn init(
        jobs: &mut ::job::Jobs,
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        deposit: &CoreDeposit<Perms, E>,
        config: CustomerSyncConfig,
    ) -> Result<Self, CustomerSyncError> {
        let keycloak_client = keycloak_client::KeycloakClient::new(config.keycloak.clone());

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_SYNC_CREATE_KEYCLOAK_USER),
                CreateKeycloakUserHandler::new(keycloak_client.clone()),
            )
            .await?;

        let sync_keycloak_email_spawner = jobs.add_initializer(
            SyncKeycloakEmailJobInitializer::new(keycloak_client.clone()),
        );

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(SYNC_EMAIL_JOB),
                SyncEmailHandler::new(sync_keycloak_email_spawner),
            )
            .await?;

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(UPDATE_LAST_ACTIVITY_DATE),
                UpdateLastActivityDateHandler::new(customers, deposit),
            )
            .await?;

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(UPDATE_CUSTOMER_ACTIVITY_STATUS),
                UpdateCustomerActivityStatusHandler::new(customers),
            )
            .await?;

        let activate_holder_account =
            jobs.add_initializer(ActivateHolderAccountJobInitializer::new(deposit.clone()));
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_ACTIVE_SYNC),
                CustomerActiveSyncHandler::new(activate_holder_account),
            )
            .await?;

        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
