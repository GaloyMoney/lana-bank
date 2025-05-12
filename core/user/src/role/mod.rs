use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

use crate::{
    primitives::{CoreUserAction, CoreUserObject},
    CoreUserEvent,
};

mod entity;
mod error;
mod repo;

use repo::RoleRepo;

#[derive(Clone)]
pub struct Roles<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreUserEvent>,
{
    authz: Perms,
    repo: RoleRepo<E>,
}

impl<Perms, E> Roles<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreUserAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreUserObject>,
    E: OutboxEventMarker<CoreUserEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        Self {
            repo: RoleRepo::new(pool),
            authz: authz.clone(),
        }
    }
}
