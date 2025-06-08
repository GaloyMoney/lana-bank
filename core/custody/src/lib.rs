#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod custodian;
pub mod error;
mod primitives;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

pub use custodian::*;

pub use config::*;
use error::CoreCustodyError;
pub use primitives::*;

#[derive(Clone)]
pub struct CoreCustody<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    custodians: CustodianRepo,
    config: CustodyConfig,
}

impl<Perms> CoreCustody<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCustodyObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, config: CustodyConfig) -> Self {
        Self {
            authz: authz.clone(),
            custodians: CustodianRepo::new(pool),
            config,
        }
    }

    #[instrument(
        name = "core_custody.create_custodian",
        skip(self, custodian_config),
        err
    )]
    pub async fn create_custodian(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: impl AsRef<str> + std::fmt::Debug,
        custodian_config: CustodianConfig,
    ) -> Result<Custodian, CoreCustodyError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_CREATE,
            )
            .await?;

        let new_custodian = NewCustodian::builder()
            .name(name.as_ref().to_owned())
            .audit_info(audit_info.clone())
            .build()
            .expect("all fields provided");
        let mut op = self.custodians.begin_op().await?;

        let mut custodian = self.custodians.create_in_op(&mut op, new_custodian).await?;

        custodian.update_custodian_config(
            custodian_config,
            &self.config.custodian_encryption.key,
            audit_info,
        )?;
        self.custodians
            .update_in_op(&mut op, &mut custodian)
            .await?;

        op.commit().await?;

        Ok(custodian)
    }

    pub fn custodian_config(&self, custodian: &Custodian) -> Option<CustodianConfig> {
        custodian.custodian_config(self.config.custodian_encryption.key)
    }

    #[instrument(name = "core_custody.find_all_custodians", skip(self), err)]
    pub async fn find_all_custodians<T: From<Custodian>>(
        &self,
        ids: &[CustodianId],
    ) -> Result<std::collections::HashMap<CustodianId, T>, CoreCustodyError> {
        Ok(self.custodians.find_all(ids).await?)
    }

    #[instrument(name = "core_custody.list_custodians", skip(self), err)]
    pub async fn list_custodians(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CustodiansByNameCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Custodian, CustodiansByNameCursor>, CoreCustodyError>
    {
        self.authz
            .enforce_permission(
                sub,
                CoreCustodyObject::all_custodians(),
                CoreCustodyAction::CUSTODIAN_LIST,
            )
            .await?;
        Ok(self
            .custodians
            .list_by_name(query, es_entity::ListDirection::Ascending)
            .await?)
    }
}
