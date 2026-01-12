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
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use obix::out::{Outbox, OutboxEventMarker};
use tracing_macros::record_error_severity;

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

        let create_keycloak_user_job_spawner =
            jobs.add_initializer(CreateKeycloakUserInit::new(outbox, keycloak_client.clone()));
        create_keycloak_user_job_spawner
            .spawn_unique(job::JobId::new(), CreateKeycloakUserJobConfig::new())
            .await?;

        let sync_email_job_spawner =
            jobs.add_initializer(SyncEmailInit::new(outbox, keycloak_client.clone()));
        sync_email_job_spawner
            .spawn_unique(job::JobId::new(), SyncEmailJobConfig::new())
            .await?;

        let update_last_activity_date_job_spawner =
            jobs.add_initializer(UpdateLastActivityDateInit::new(outbox, customers, deposit));
        update_last_activity_date_job_spawner
            .spawn_unique(job::JobId::new(), UpdateLastActivityDateConfig::new())
            .await?;

        let update_customer_activity_status_job_spawner = jobs.add_initializer(
            UpdateCustomerActivityStatusInit::new(customers, config.clone()),
        );
        update_customer_activity_status_job_spawner
            .spawn_unique(
                job::JobId::new(),
                UpdateCustomerActivityStatusJobConfig::new(),
            )
            .await?;

        let customer_active_sync_job_spawner =
            jobs.add_initializer(CustomerActiveSyncInit::new(outbox, deposit, config));
        customer_active_sync_job_spawner
            .spawn_unique(job::JobId::new(), CustomerActiveSyncJobConfig::new())
            .await?;

        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
