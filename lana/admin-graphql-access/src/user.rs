use async_graphql::*;

use admin_graphql_shared::primitives::*;

use super::User;

#[derive(InputObject)]
pub struct UserCreateInput {
    pub email: String,
    pub role_id: UUID,
}

mutation_payload! { UserCreatePayload, user: User }

#[derive(InputObject)]
pub struct UserUpdateRoleInput {
    pub id: UUID,
    pub role_id: UUID,
}
mutation_payload! { UserUpdateRolePayload, user: User }
