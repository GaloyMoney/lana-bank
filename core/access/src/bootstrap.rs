use std::collections::{HashMap, HashSet};

use audit::SystemSubject;
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
    authz: Authorization<Audit, AuthRoleToken>,
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
        authz: &Authorization<Audit, AuthRoleToken>,
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
    /// User with `email` will have a role “superuser” that has all available permission sets.
    pub(super) async fn bootstrap_access_control(
        &self,
        email: String,
        actions: Vec<ActionMapping>,
        predefined_roles: &[(&'static str, &[&'static str])],
    ) -> Result<(), CoreAccessError> {
        let mut db = self.role_repo.begin_op().await?;

        let permission_sets = self.bootstrap_permission_sets(&mut db, &actions).await?;
        let superuser_role = self
            .bootstrap_roles(&mut db, &permission_sets, predefined_roles)
            .await?;
        let superuser = self
            .users
            .bootstrap_superuser_user(&mut db, email, &superuser_role)
            .await?;

        self.authz
            .assign_role_to_subject(superuser.id, superuser_role.id)
            .await?;

        db.commit().await?;

        // Subject::System also has the superuser role
        self.authz
            .assign_role_to_subject(
                <<Audit as AuditSvc>::Subject as SystemSubject>::system(),
                superuser_role.id,
            )
            .await?;

        Ok(())
    }

    async fn create_role(
        &self,
        db: &mut DbOp<'_>,
        name: String,
        permission_sets: HashSet<PermissionSetId>,
        audit_info: &AuditInfo,
    ) -> Result<Role, RoleError> {
        let existing = self.role_repo.find_by_name(&name).await;
        let role = if matches!(existing, Err(ref e) if e.was_not_found()) {
            let new_role = NewRole::builder()
                .id(RoleId::new())
                .name(name)
                .audit_info(audit_info.clone())
                .initial_permission_sets(permission_sets.clone())
                .build()
                .expect("all fields for new role provided");

            self.role_repo.create_in_op(db, new_role).await?
        } else {
            existing?
        };

        for permission_set_id in permission_sets {
            self.authz
                .add_role_hierarchy(role.id, permission_set_id)
                .await?;
        }

        Ok(role)
    }

    /// Creates a role with name “superuser” that will have all given permission sets.
    /// Used for bootstrapping the application.
    ///
    /// Also creates roles for all predefined roles.
    async fn bootstrap_roles(
        &self,
        db: &mut DbOp<'_>,
        permission_sets: &[PermissionSet],
        predefined_roles: &[(&'static str, &[&'static str])],
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

        let all_permission_sets = permission_sets
            .iter()
            .map(|ps| (ps.name.clone(), ps.id))
            .collect::<HashMap<_, _>>();

        let superuser_role = self
            .create_role(
                db,
                ROLE_NAME_SUPERUSER.to_owned(),
                all_permission_sets.values().copied().collect(),
                &audit_info,
            )
            .await?;

        for (name, sets) in predefined_roles {
            let sets = sets
                .iter()
                .map(|set| all_permission_sets.get(*set).unwrap())
                .copied()
                .collect::<HashSet<_>>();

            let _ = self
                .create_role(db, name.to_string(), sets, &audit_info)
                .await;
        }

        Ok(superuser_role)
    }

    /// Generates Permission Sets based on provided hierarchy of modules and
    /// returns all existing Permission Sets. For use during application bootstrap.
    async fn bootstrap_permission_sets(
        &self,
        db: &mut DbOp<'_>,
        actions: &[ActionMapping],
    ) -> Result<Vec<PermissionSet>, PermissionSetError> {
        let existing = self
            .permission_set_repo
            .list_by_id(Default::default(), Default::default())
            .await?
            .entities
            .into_iter()
            .map(|ps| (ps.name.to_string(), ps))
            .collect::<HashMap<_, _>>();

        #[allow(clippy::type_complexity)]
        let mut permission_sets: HashMap<
            &'static str,
            Vec<Permission<Audit::Object, Audit::Action>>,
        > = Default::default();

        for action in actions {
            let set = action.permission_set();
            permission_sets.entry(set).or_default().push(action.into());
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

        for permission_set in &new {
            for permission in permission_set.permissions() {
                self.authz
                    .add_permission_to_role(
                        &permission_set.id,
                        permission.object(),
                        permission.action(),
                    )
                    .await?;
            }
        }

        Ok(existing.into_values().chain(new.into_iter()).collect())
    }
}
