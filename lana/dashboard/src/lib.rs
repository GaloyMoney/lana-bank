#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
mod jobs;
mod primitives;
mod repo;
mod values;

use sqlx::PgPool;

use audit::AuditSvc;
use authz::PermissionCheck;
use obix::out::OutboxEventJobConfig;
use tracing_macros::record_error_severity;

use error::*;
use jobs::*;
pub use primitives::*;
use repo::*;
pub use values::*;

pub type Outbox = obix::Outbox<lana_events::LanaEvent>;

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
    #[record_error_severity]
    #[tracing::instrument(name = "dashboard.init", skip_all)]
    pub async fn init(
        pool: &PgPool,
        authz: &Perms,
        jobs: &mut ::job::Jobs,
        outbox: &Outbox,
    ) -> Result<Self, DashboardError> {
        let repo = DashboardRepo::new(pool);

        let update_dashboard =
            jobs.add_initializer(UpdateDashboardJobInitializer::new(repo.clone()));
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(DASHBOARD_PROJECTION_JOB),
                DashboardProjectionHandler::new(update_dashboard),
            )
            .await?;

        Ok(Self {
            _outbox: outbox.clone(),
            authz: authz.clone(),
            repo,
        })
    }

    #[record_error_severity]
    #[tracing::instrument(name = "dashboard.load", skip(self), fields(subject = %sub))]
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
