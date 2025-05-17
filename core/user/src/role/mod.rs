use audit::AuditSvc;
use authz::{Authorization, PermissionCheck as _};
use es_entity::DbOp;
use outbox::OutboxEventMarker;

use crate::{
    event::CoreUserEvent,
    primitives::{CoreUserAction, CoreUserObject, RoleId},
    publisher::UserPublisher,
    PermissionSetId, RoleName,
};

mod entity;
pub mod error;
mod repo;

pub use entity::{NewRole, Role, RoleEvent};
pub use error::RoleError;
use repo::RoleRepo;

pub struct Roles<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    authz: Authorization<Audit, RoleName>,
    pub(super) repo: RoleRepo<E>,
}

impl<Audit, E> Roles<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Action: From<CoreUserAction>,
    <Audit as AuditSvc>::Object: From<CoreUserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Authorization<Audit, RoleName>,
        publisher: &UserPublisher<E>,
    ) -> Self {
        Self {
            repo: RoleRepo::new(pool, publisher),
            authz: authz.clone(),
        }
    }

    pub async fn find_by_id(&self, role_id: RoleId) -> Result<Role, RoleError> {
        self.repo.find_by_id(&role_id).await
    }

    pub async fn list(&self, sub: &<Audit as AuditSvc>::Subject) -> Result<Vec<Role>, RoleError> {
        Ok(self
            .repo
            .list_by_created_at(Default::default(), Default::default())
            .await?
            .entities)
    }

    pub async fn update(&self, role: &mut Role) -> Result<(), RoleError> {
        self.repo.update(role).await?;
        Ok(())
    }

    /// Creates a new role with a given name. The names must be unique,
    /// an error will be raised in case of conflict. If `base_role` is provided,
    /// the new role will have all its permission sets.
    pub async fn create_role(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        name: RoleName,
        base_role: Option<RoleId>,
    ) -> Result<Role, RoleError> {
        let permission_sets = match base_role {
            None => vec![],
            _ => vec![],
        };

        self.create_role_with_permissions_sets(sub, name, &permission_sets)
            .await
    }

    pub async fn create_role_with_permissions_sets(
        &self,
        sub: &<Audit as AuditSvc>::Subject,
        name: RoleName,
        permission_sets: &[PermissionSetId],
    ) -> Result<Role, RoleError> {
        self.authz
            .enforce_permission(
                sub,
                CoreUserObject::all_roles(),
                CoreUserAction::ROLE_CREATE,
            )
            .await?;

        let new_role = NewRole::builder()
            .id(RoleId::new())
            .name(name)
            .permission_sets(permission_sets.iter().copied().collect())
            .build()
            .expect("all fields for new role provided");

        let role = self.repo.create(new_role).await?;

        Ok(role)
    }

    /// Creates a role with name “superuser” that will have all given permission sets.
    /// Used for bootstrapping the application.
    //
    // Warning: think thrice if you need to make the method more visible.
    pub(super) async fn bootstrap_superuser(
        &self,
        permission_sets: &[PermissionSetId],
        db: &mut DbOp<'_>,
    ) -> Result<Role, RoleError> {
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreUserObject::all_users(),
                CoreUserAction::ROLE_CREATE,
            )
            .await?;

        // TODO check that role does not exist in DB yet

        let new_role = NewRole::builder()
            .id(RoleId::new())
            .name(RoleName::SUPERUSER)
            .audit_info(audit_info)
            .permission_sets(permission_sets.iter().copied().collect())
            .build()
            .expect("all fields for new role provided");

        self.repo.create_in_op(db, new_role).await
    }
}

impl<Audit, E> Clone for Roles<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreUserEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}
