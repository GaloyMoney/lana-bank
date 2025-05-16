//! _Permission Set_ is a predefined named set of permissions. Administrators with sufficient
//! permissions can assign Permission Sets to a [Role](super::role) and thus give the users
//! with this role all permissions of the Permission Set.
//!
//! The main purpose of Permission Sets is to group related permissions under a common name and
//! shield the administrator from actual permissions that can be too dynamic and have too high a granularity.
//! Permission Sets are not intended to be created or deleted in a running application; they are expected
//! to be created and defined during application bootstrap and remain unchanged for their entire life.

use std::collections::{HashMap, HashSet};

use audit::AuditSvc;
use authz::Authorization;
use entity::NewPermissionSet;
use es_entity::DbOp;

use crate::{
    primitives::{CoreUserAction, CoreUserObject},
    PermissionSetId, RoleName,
};

mod entity;
mod error;
mod repo;

pub use entity::PermissionSet;
pub use error::PermissionSetError;
use repo::PermissionSetRepo;

pub struct PermissionSets<Audit>
where
    Audit: AuditSvc,
{
    authz: Authorization<Audit, RoleName>,
    repo: PermissionSetRepo,
}

impl<Audit> PermissionSets<Audit>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Action: From<CoreUserAction>,
    <Audit as AuditSvc>::Object: From<CoreUserObject>,
{
    pub fn new(authz: &Authorization<Audit, RoleName>, pool: &sqlx::PgPool) -> Self {
        Self {
            authz: authz.clone(),
            repo: PermissionSetRepo::new(pool),
        }
    }

    pub async fn find_by_id(
        &self,
        id: PermissionSetId,
    ) -> Result<PermissionSet, PermissionSetError> {
        self.repo.find_by_id(id).await
    }

    pub async fn find_all(
        &self,
        ids: &[PermissionSetId],
    ) -> Result<HashMap<PermissionSetId, PermissionSet>, PermissionSetError> {
        self.repo.find_all(ids).await
    }

    pub async fn list(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
    ) -> Result<Vec<PermissionSet>, PermissionSetError> {
        Ok(vec![])
    }

    pub(super) async fn bootstrap_permission_sets(
        &self,
        db: &mut DbOp<'_>,
    ) -> Result<Vec<PermissionSet>, PermissionSetError> {
        let permissions = HashSet::from([("abc/def/*".to_string(), "abd:def:update".to_string())]);

        let new_permission_set = NewPermissionSet {
            id: PermissionSetId::new(),
            name: "User Manager".to_string(),
            permissions,
        };

        let permission_set = self.repo.create_in_op(db, new_permission_set).await?;

        Ok(vec![permission_set])
    }
}

impl<Audit> Clone for PermissionSets<Audit>
where
    Audit: AuditSvc,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}
