#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod error;
mod job;
mod time;

use config::*;
use error::*;
use job::*;

use audit::AuditSvc;
use core_user::{CoreUserAction, CoreUserEvent, UserId, UserObject, Users};
use outbox::{Outbox, OutboxEventMarker};

pub struct UserOnboarding<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    _phantom: std::marker::PhantomData<(Audit, E)>,
    _outbox: Outbox<E>,
}

impl<Audit, E> Clone for UserOnboarding<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
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
    <Audit as AuditSvc>::Action: From<CoreUserAction>,
    <Audit as AuditSvc>::Object: From<UserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub async fn init(
        jobs: &::job::Jobs,
        outbox: &Outbox<E>,
        users: &Users<Audit, E>,
        config: UserOnboardingConfig,
    ) -> Result<Self, UserOnboardingError> {
        let kratos_admin = kratos_admin::KratosAdmin::init(config.kratos_admin);

        jobs.add_initializer_and_spawn_unique(
            UserOnboardingJobInitializer::new(outbox, users, kratos_admin),
            UserOnboardingJobConfig::new(),
        )
        .await?;
        Ok(Self {
            _phantom: std::marker::PhantomData,
            _outbox: outbox.clone(),
        })
    }
}
