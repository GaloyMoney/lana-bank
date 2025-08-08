#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod error;
mod job;

use config::UserOnboardingConfig;
use error::*;
use job::*;

use audit::AuditSvc;
use core_access::{CoreAccessAction, CoreAccessEvent, CoreAccessObject, UserId, user::Users};
use outbox::{Outbox, OutboxEventMarker};

pub struct UserOnboarding<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    _phantom: std::marker::PhantomData<(Audit, E)>,
    _outbox: Outbox<E>,
}

impl<Audit, E> Clone for UserOnboarding<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Audit, E> UserOnboarding<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub async fn init(
        jobs: &::job::Jobs,
        outbox: &Outbox<E>,
        users: &Users<Audit, E>,
        config: UserOnboardingConfig,
    ) -> Result<Self, UserOnboardingError> {
        let keycloak_client = keycloak_client::KeycloakClient::new(config.keycloak);

        jobs.add_initializer_and_spawn_unique(
            UserOnboardingInit::new(outbox, users, keycloak_client),
            UserOnboardingJobConfig::new(),
        )
        .await?;
        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
