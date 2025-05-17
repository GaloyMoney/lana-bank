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
use authz::{
    permission_set::{ActionDescription, FullPath},
    Authorization,
};
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

    /// Generates Permission Sets based on provided hierarchy of modules and
    /// returns all existing Permission Sets. For use during application bootstrap.
    //
    // Warning: think thrice if you need to make the method more visible.
    pub(super) async fn bootstrap_permission_sets(
        &self,
        actions: &[ActionDescription<FullPath>],
        db: &mut DbOp<'_>,
    ) -> Result<Vec<PermissionSet>, PermissionSetError> {
        // TODO: Handle those already existing but always return all.

        let mut permission_sets: HashMap<&'static str, HashSet<(String, String)>> =
            Default::default();

        for action in actions {
            for set in action.permission_sets() {
                permission_sets
                    .entry(*set)
                    .or_default()
                    .insert((action.all_objects_name(), action.action_name()));
            }
        }

        let new_permission_sets = permission_sets
            .into_iter()
            .map(|(set, permissions)| NewPermissionSet {
                id: PermissionSetId::new(),
                name: set.to_string(),
                permissions,
            })
            .collect();

        self.repo.create_all_in_op(db, new_permission_sets).await
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
