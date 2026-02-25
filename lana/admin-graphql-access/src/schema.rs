use async_graphql::{Context, Object, types::connection::*};

use admin_graphql_shared::primitives::UUID;

use crate::permission_set::*;
use crate::role::*;
use crate::user::*;

#[derive(Default)]
pub struct AccessQuery;

#[Object]
impl AccessQuery {
    async fn user(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<User>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(User, app.access().users().find_by_id(sub, id))
    }

    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let users: Vec<_> = app
            .access()
            .users()
            .list_users(sub)
            .await?
            .into_iter()
            .map(User::from)
            .collect();
        Ok(users)
    }

    async fn role(&self, ctx: &Context<'_>, id: UUID) -> async_graphql::Result<Option<Role>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(Role, app.access().find_role_by_id(sub, id))
    }

    async fn roles(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<Connection<RolesByNameCursor, Role, EmptyFields, EmptyFields>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(RolesByNameCursor, Role, after, first, |query| app
            .access()
            .list_roles(sub, query))
    }

    async fn permission_sets(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<PermissionSetsByIdCursor, PermissionSet, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            PermissionSetsByIdCursor,
            PermissionSet,
            after,
            first,
            |query| app.access().list_permission_sets(sub, query)
        )
    }
}

#[derive(Default)]
pub struct AccessMutation;

#[Object]
impl AccessMutation {
    async fn user_create(
        &self,
        ctx: &Context<'_>,
        input: UserCreateInput,
    ) -> async_graphql::Result<UserCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            UserCreatePayload,
            User,
            app.access().create_user(sub, input.email, input.role_id)
        )
    }

    async fn user_update_role(
        &self,
        ctx: &Context<'_>,
        input: UserUpdateRoleInput,
    ) -> async_graphql::Result<UserUpdateRolePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let UserUpdateRoleInput { id, role_id } = input;
        exec_mutation!(
            UserUpdateRolePayload,
            User,
            app.access().update_role_of_user(sub, id, role_id)
        )
    }

    async fn role_create(
        &self,
        ctx: &Context<'_>,
        input: RoleCreateInput,
    ) -> async_graphql::Result<RoleCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let RoleCreateInput {
            name,
            permission_set_ids,
        } = input;
        exec_mutation!(
            RoleCreatePayload,
            Role,
            app.access().create_role(sub, name, permission_set_ids)
        )
    }

    async fn role_add_permission_sets(
        &self,
        ctx: &Context<'_>,
        input: RoleAddPermissionSetsInput,
    ) -> async_graphql::Result<RoleAddPermissionSetsPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            RoleAddPermissionSetsPayload,
            Role,
            app.access()
                .add_permission_sets_to_role(sub, input.role_id, input.permission_set_ids)
        )
    }

    async fn role_remove_permission_sets(
        &self,
        ctx: &Context<'_>,
        input: RoleRemovePermissionSetsInput,
    ) -> async_graphql::Result<RoleRemovePermissionSetsPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            RoleRemovePermissionSetsPayload,
            Role,
            app.access().remove_permission_sets_from_role(
                sub,
                input.role_id,
                input.permission_set_ids
            )
        )
    }
}
