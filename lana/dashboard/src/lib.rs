#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
mod job;
mod primitives;
mod repo;
mod time;
mod values;

use sqlx::PgPool;

use audit::AuditSvc;
use authz::PermissionCheck;
use lana_events::LanaEvent;

use error::*;
use job::*;
pub use primitives::*;
use repo::*;
pub use values::*;

pub type Outbox = outbox::Outbox<LanaEvent>;

pub struct Dashboard<Perms>
where
    Perms: PermissionCheck,
{
    _outbox: Outbox,
    authz: Perms,
    repo: DashboardRepo,
}

impl<Perms: PermissionCheck> Clone for Dashboard<Perms> {
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Perms> Dashboard<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<DashboardModuleAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<DashboardModuleObject>,
{
    pub async fn init(
        pool: &PgPool,
        authz: &Perms,
        jobs: &::job::Jobs,
        outbox: &Outbox,
    ) -> Result<Self, DashboardError> {
        let repo = DashboardRepo::new(pool);
        jobs.add_initializer_and_spawn_unique(
            DashboardProjectionJobInitializer::new(outbox, &repo),
            DashboardProjectionJobConfig,
        )
        .await?;
        Ok(Self {
            _outbox: outbox.clone(),
            authz: authz.clone(),
            repo,
        })
    }

    pub async fn load(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<DashboardValues, DashboardError> {
        self.authz
            .enforce_permission(
                sub,
                DashboardModuleObject::all_dashboards(),
                DashboardModuleAction::DASHBOARD_READ,
            )
            .await?;
        let res = self.repo.load().await?;
        Ok(res)
    }
}
