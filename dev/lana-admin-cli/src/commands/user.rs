use anyhow::Result;

use crate::cli::UserAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: UserAction, json: bool) -> Result<()> {
    match action {
        UserAction::RolesList => {
            let vars = roles_list::Variables {};
            let data = client.execute::<RolesList>(vars).await?;
            let nodes = data.roles.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|r| vec![r.role_id.clone(), r.name.clone()])
                    .collect();
                output::print_table(&["Role ID", "Name"], rows);
            }
        }
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
    }
    Ok(())
}
