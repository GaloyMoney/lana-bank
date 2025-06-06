#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod custodian;
pub mod error;
mod primitives;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

pub use custodian::*;

use error::CoreCustodyError;
pub use primitives::*;

#[derive(Clone)]
pub struct CoreCustody<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    custodians: CustodianRepo,
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
            custodians: CustodianRepo::new(pool),
        }
    }

    #[instrument(
        name = "core_custody.create_custodian_config",
        skip(self, custodian),
        err
    )]
    pub async fn create_custodian_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: impl AsRef<str> + std::fmt::Debug,
        custodian: CustodianConfig,
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
            .custodian(custodian)
            .audit_info(audit_info)
            .build()
            .expect("all fields provided");

        Ok(self.custodians.create(new_custodian).await?)
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
