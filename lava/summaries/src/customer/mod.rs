mod error;
mod job;
mod primitives;
mod repo;
mod values;

use sqlx::PgPool;

use audit::AuditSvc;
use authz::PermissionCheck;

use crate::Outbox;

use error::*;
use job::*;
use primitives::*;
use repo::*;
use values::*;

pub struct CustomerSummaries<Perms>
where
    Perms: PermissionCheck,
{
    _outbox: Outbox,
    authz: Perms,
    repo: CustomerSummaryRepo,
}

impl<Perms: PermissionCheck> Clone for CustomerSummaries<Perms> {
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Perms> CustomerSummaries<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CustomerSummaryModuleAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerSummaryModuleObject>,
{
    pub async fn init(
        pool: &PgPool,
        authz: &Perms,
        jobs: &::job::Jobs,
        outbox: &Outbox,
    ) -> Result<Self, CustomerSummaryError> {
        let repo = CustomerSummaryRepo::new(pool);
        jobs.add_initializer_and_spawn_unique(
            CustomerSummaryProjectionJobInitializer::new(outbox, &repo),
            CustomerSummaryProjectionJobConfig,
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
        id: impl Into<uuid::Uuid>,
    ) -> Result<CustomerSummaryValues, CustomerSummaryError> {
        self.authz
            .enforce_permission(
                sub,
                CustomerSummaryModuleObject::CustomerSummary,
                CustomerSummaryModuleAction::CUSTOMER_SUMMARY_READ,
            )
            .await?;
        let res = self.repo.load(id.into()).await?;
        Ok(res)
    }
}
