use anyhow::Result;

use crate::cli::UserAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: UserAction, json: bool) -> Result<()> {
    match action {
        UserAction::Create { email, role_id } => {
            let vars = user_create::Variables {
                input: user_create::UserCreateInput { email, role_id },
            };
            let data = client.execute::<UserCreate>(vars).await?;
            let u = data.user_create.user;
            if json {
                output::print_json(&u)?;
            } else {
                output::print_kv(&[
                    ("User ID", &u.user_id),
                    ("Email", &u.email),
                    ("Role", &u.role.name),
                ]);
            }
        }
        UserAction::UpdateRole { user_id, role_id } => {
            let vars = user_update_role::Variables {
                input: user_update_role::UserUpdateRoleInput { user_id, role_id },
            };
            let data = client.execute::<UserUpdateRole>(vars).await?;
            let u = data.user_update_role.user;
            if json {
                output::print_json(&u)?;
            } else {
                output::print_kv(&[
                    ("User ID", &u.user_id),
                    ("Email", &u.email),
                    ("Role", &u.role.name),
                ]);
            }
        }
    }
    Ok(())
}
