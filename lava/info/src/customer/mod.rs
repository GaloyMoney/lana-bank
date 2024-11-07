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

pub struct CustomerInfos<Perms>
where
    Perms: PermissionCheck,
{
    _outbox: Outbox,
    authz: Perms,
    repo: CustomerInfoRepo,
}

impl<Perms: PermissionCheck> Clone for CustomerInfos<Perms> {
    fn clone(&self) -> Self {
        Self {
            _outbox: self._outbox.clone(),
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Perms> CustomerInfos<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CustomerInfoModuleAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerInfoModuleObject>,
{
    pub async fn init(
        pool: &PgPool,
        authz: &Perms,
        jobs: &::job::Jobs,
        outbox: &Outbox,
    ) -> Result<Self, CustomerInfoError> {
        let repo = CustomerInfoRepo::new(pool);
        jobs.add_initializer_and_spawn_unique(
            CustomerInfoProjectionJobInitializer::new(outbox, &repo),
            CustomerInfoProjectionJobConfig,
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
    ) -> Result<CustomerInfoValues, CustomerInfoError> {
        self.authz
            .enforce_permission(
                sub,
                CustomerInfoModuleObject::CustomerInfo,
                CustomerInfoModuleAction::CUSTOMER_INFO_READ,
            )
            .await?;
        let res = self.repo.load(id.into()).await?;
        Ok(res)
    }
}
