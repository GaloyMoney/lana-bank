#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod custodian_config;
mod primitives;

use audit::AuditSvc;
use authz::PermissionCheck;

use custodian_config::CustodianConfigRepo;

pub use primitives::*;

pub struct CoreCustody<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    custodian_configs: CustodianConfigRepo,
}

impl<Perms> CoreCustody<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        Self {
            authz: authz.clone(),
            custodian_configs: CustodianConfigRepo::new(pool),
        }
    }

    pub async fn create_custodian_config(&self) {}

    pub async fn list_custodian_configs(&self) {}
}
