#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod error;
mod job;

use config::UserOnboardingConfig;
use error::*;
use job::*;

use core_access::CoreAccessEvent;
use obix::out::{Outbox, OutboxEventMarker};
use tracing_macros::record_error_severity;

pub struct UserOnboarding<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    _phantom: std::marker::PhantomData<E>,
    _outbox: Outbox<E>,
}

impl<E> Clone for UserOnboarding<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> UserOnboarding<E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "user_onboarding.init", skip_all)]
    pub async fn init(
        jobs: &::job::Jobs,
        outbox: &Outbox<E>,
        config: UserOnboardingConfig,
    ) -> Result<Self, UserOnboardingError> {
        let keycloak_client = keycloak_client::KeycloakClient::new(config.keycloak);

        jobs.add_initializer_and_spawn_unique(
            UserOnboardingInit::new(outbox, keycloak_client),
            UserOnboardingJobConfig::new(),
        )
        .await?;
        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
