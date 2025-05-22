use std::collections::{HashMap, HashSet};

use authz::action_description::*;
use es_entity::DbOp;

use crate::{
    error::CoreAccessError,
    permission_set::{NewPermissionSet, PermissionSet, PermissionSetError},
    *,
};

pub(super) struct Bootstrap<Audit, E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
    Audit: AuditSvc,
{
    authz: Authorization<Audit, RoleName>,
    role_repo: RoleRepo<E>,
    permission_set_repo: PermissionSetRepo,
    users: Users<Audit, E>,
}

impl<Audit, E> Bootstrap<Audit, E>
where
    E: OutboxEventMarker<CoreAccessEvent>,
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
{
    pub(super) fn new(
        authz: &Authorization<Audit, RoleName>,
        role_repo: &RoleRepo<E>,
        users: &Users<Audit, E>,
        permission_set_repo: &PermissionSetRepo,
    ) -> Self {
        Self {
            authz: authz.clone(),
            role_repo: role_repo.clone(),
            permission_set_repo: permission_set_repo.clone(),
            users: users.clone(),
        }
    }

    /// Creates essential users, roles and permission sets for a running application.
    /// Without these, seeding of other roles cannot be initiated. User with `email`
    /// will have a role “superuser” that has all available permission sets.
    pub(super) async fn bootstrap_access_control(
        &self,
        email: String,
        actions: &[ActionDescription<FullPath>],
    ) -> Result<(), CoreAccessError> {
        let mut db = self.role_repo.begin_op().await?;

        let permission_sets = self.bootstrap_permission_sets(&mut db, actions).await?;

        let permission_set_ids = permission_sets.iter().map(|s| s.id).collect::<Vec<_>>();
        let superuser_role = self
            .bootstrap_superuser_role(&mut db, &permission_set_ids)
            .await?;
        let superuser = self
            .users
            .bootstrap_superuser_user(&mut db, email, superuser_role.name)
            .await?;

        self.authz
            .assign_role_to_subject(superuser.id, RoleName::SUPERUSER)
            .await?;

        db.commit().await?;

        Ok(())
    }

    /// Creates a role with name “superuser” that will have all given permission sets.
    /// Used for bootstrapping the application.
    async fn bootstrap_superuser_role(
        &self,
        db: &mut DbOp<'_>,
        permission_sets: &[PermissionSetId],
    ) -> Result<Role, RoleError> {
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreAccessObject::all_users(),
                CoreAccessAction::ROLE_CREATE,
            )
            .await?;

        let existing_role = self
            .role_repo
            .find_by_name_in_tx(db.tx(), &RoleName::SUPERUSER)
            .await;

        if matches!(existing_role, Err(ref e) if e.was_not_found()) {
            let new_role = NewRole::builder()
                .id(RoleId::new())
                .name(RoleName::SUPERUSER)
                .audit_info(audit_info)
                .permission_sets(permission_sets.iter().copied().collect())
                .build()
                .expect("all fields for new role provided");

            self.role_repo.create_in_op(db, new_role).await
        } else {
            existing_role
        }
    }

    /// Generates Permission Sets based on provided hierarchy of modules and
    /// returns all existing Permission Sets. For use during application bootstrap.
    async fn bootstrap_permission_sets(
        &self,
        db: &mut DbOp<'_>,
        actions: &[ActionDescription<FullPath>],
    ) -> Result<Vec<PermissionSet>, PermissionSetError> {
        let existing = self
            .permission_set_repo
            .list_by_id(Default::default(), Default::default())
            .await?
            .entities
            .into_iter()
            .map(|ps| (ps.name.to_string(), ps))
            .collect::<HashMap<_, _>>();

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

        // Create only those permission sets that do not exist yet. Don't remove anything.
        permission_sets.retain(|k, _| !existing.contains_key(*k));

        let new_permission_sets = permission_sets
            .into_iter()
            .map(|(set, permissions)| {
                NewPermissionSet::builder()
                    .id(PermissionSetId::new())
                    .name(set)
                    .permissions(permissions)
                    .build()
                    .expect("all fields for new permission set provided")
            })
            .collect::<Vec<_>>();

        let new = if new_permission_sets.is_empty() {
            vec![]
        } else {
            self.permission_set_repo
                .create_all_in_op(db, new_permission_sets)
                .await?
        };

        Ok(existing.into_values().chain(new.into_iter()).collect())
    }
}
