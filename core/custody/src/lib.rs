#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod custodian_config;
pub mod error;
mod primitives;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use custodian_config::{Custodian, CustodianConfig, CustodianConfigRepo, NewCustodianConfig};

use error::CoreCustodyError;
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

    #[instrument(name = "core_custody.create_custodian_config", skip(self, custodian), err)]
    pub async fn create_custodian_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: impl AsRef<str> + std::fmt::Debug,
        custodian: Custodian,
    ) -> Result<CustodianConfig, CoreCustodyError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodian_configs(),
                CoreCustodyAction::CUSTODIAN_CONFIG_CREATE,
            )
            .await?;

        let new_custodian_config = NewCustodianConfig::builder()
            .name(name.as_ref().to_owned())
            .custodian(custodian)
            .audit_info(audit_info)
            .build()
            .expect("all fields provided");

        Ok(self.custodian_configs.create(new_custodian_config).await?)
    }

    pub async fn list_custodian_configs(&self) {}
}
