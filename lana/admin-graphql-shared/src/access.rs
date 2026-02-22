use async_graphql::*;

use crate::primitives::*;
use lana_app::access::permission_set::PermissionSet as DomainPermissionSet;
use lana_app::access::role::Role as DomainRole;
use lana_app::access::user::User as DomainUser;

// PermissionSet

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PermissionSet {
    id: ID,
    permission_set_id: UUID,

    #[graphql(skip)]
    pub entity: Arc<DomainPermissionSet>,
}

#[ComplexObject]
impl PermissionSet {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn description(&self) -> &str {
        permission_sets_macro::find_by_name(&self.entity.name)
            .map(|e| e.description)
            .unwrap_or("")
    }
}

impl From<DomainPermissionSet> for PermissionSet {
    fn from(permission_set: DomainPermissionSet) -> Self {
        Self {
            id: permission_set.id.to_global_id(),
            permission_set_id: UUID::from(permission_set.id),
            entity: Arc::new(permission_set),
        }
    }
}

// Role

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Role {
    id: ID,
    role_id: UUID,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainRole>,
}

#[ComplexObject]
impl Role {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn permission_sets(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<PermissionSet>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let ids: Vec<_> = self.entity.permission_sets.iter().copied().collect();
        let loaded: std::collections::HashMap<PermissionSetId, PermissionSet> =
            app.access().find_all_permission_sets(&ids).await?;
        Ok(loaded.into_values().collect())
    }
}

impl From<DomainRole> for Role {
    fn from(role: DomainRole) -> Self {
        Self {
            id: role.id.to_global_id(),
            role_id: UUID::from(role.id),
            created_at: role.created_at().into(),
            entity: Arc::new(role),
        }
    }
}

// User

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct User {
    id: ID,
    user_id: UUID,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainUser>,
}

impl From<DomainUser> for User {
    fn from(user: DomainUser) -> Self {
        Self {
            id: user.id.to_global_id(),
            user_id: UUID::from(user.id),
            created_at: user.created_at().into(),
            entity: Arc::new(user),
        }
    }
}

impl From<Arc<DomainUser>> for User {
    fn from(user: Arc<DomainUser>) -> Self {
        Self {
            id: user.id.to_global_id(),
            user_id: UUID::from(user.id),
            created_at: user.created_at().into(),
            entity: user,
        }
    }
}

#[ComplexObject]
impl User {
    async fn role(&self, ctx: &Context<'_>) -> async_graphql::Result<Role> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let role_id = self.entity.current_role();
        let role = app
            .access()
            .find_role_by_id(sub, role_id)
            .await?
            .ok_or_else(|| {
                Error::new(format!(
                    "Data integrity error: Role with ID {} not found for user {}. This should never happen.",
                    role_id, self.entity.id
                ))
            })?;
        Ok(Role::from(role))
    }

    async fn email(&self) -> &str {
        &self.entity.email
    }

    async fn user_can_update_role_of_user(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .access()
            .users()
            .subject_can_update_role_of_user(sub, None, false)
            .await
            .is_ok())
    }
}
