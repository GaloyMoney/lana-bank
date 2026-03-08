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
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
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
        deposit: &CoreDeposit<Perms, E>,
        config: CustomerSyncConfig,
        sumsub_client: sumsub::SumsubClient,
    ) -> Result<Self, CustomerSyncError> {
        let keycloak_client = keycloak_client::KeycloakClient::new(config.keycloak.clone());

        let create_keycloak_user_spawner = jobs.add_initializer(
            CreateKeycloakUserJobInitializer::new(keycloak_client.clone()),
        );

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_SYNC_CREATE_KEYCLOAK_USER),
                SyncPartyKeycloakHandler::new(create_keycloak_user_spawner),
            )
            .await?;

        let disable_keycloak_user_spawner = jobs.add_initializer(
            DisableKeycloakUserJobInitializer::new(keycloak_client.clone()),
        );

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_SYNC_DISABLE_KEYCLOAK_USER),
                SyncCustomerCloseKeycloakHandler::new(disable_keycloak_user_spawner),
            )
            .await?;

        let update_user_email_spawner =
            jobs.add_initializer(UpdateUserEmailJobInitializer::new(keycloak_client.clone()));

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(SYNC_EMAIL_JOB),
                SyncEmailHandler::new(update_user_email_spawner),
            )
            .await?;

        let freeze_customer_deposits_spawner = jobs.add_initializer(
            FreezeCustomerDepositsJobInitializer::new(deposit.clone(), keycloak_client.clone()),
        );
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_FREEZE_SYNC),
                SyncCustomerFreezeHandler::new(freeze_customer_deposits_spawner),
            )
            .await?;

        let unfreeze_customer_deposits_spawner = jobs.add_initializer(
            UnfreezeCustomerDepositsJobInitializer::new(deposit.clone(), keycloak_client.clone()),
        );
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_UNFREEZE_SYNC),
                SyncCustomerUnfreezeHandler::new(unfreeze_customer_deposits_spawner),
            )
            .await?;

        let reject_sumsub_applicant_spawner = jobs.add_initializer(
            RejectSumsubApplicantJobInitializer::new(sumsub_client.clone()),
        );
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_SYNC_REJECT_SUMSUB_APPLICANT),
                SyncCustomerFrozenSumsubHandler::new(reject_sumsub_applicant_spawner),
            )
            .await?;

        let approve_sumsub_applicant_spawner = jobs.add_initializer(
            ApproveSumsubApplicantJobInitializer::new(sumsub_client.clone()),
        );
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_SYNC_APPROVE_SUMSUB_APPLICANT),
                SyncCustomerUnfrozenSumsubHandler::new(approve_sumsub_applicant_spawner),
            )
            .await?;

        let deactivate_sumsub_applicant_spawner =
            jobs.add_initializer(DeactivateSumsubApplicantJobInitializer::new(sumsub_client));
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(CUSTOMER_SYNC_DEACTIVATE_SUMSUB_APPLICANT),
                SyncCustomerFreezeSumsubHandler::new(deactivate_sumsub_applicant_spawner),
            )
            .await?;

        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
