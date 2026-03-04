use anyhow::Result;

use crate::cli::RoleAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: RoleAction, json: bool) -> Result<()> {
    match action {
        RoleAction::List { first, after } => {
            let vars = roles_list::Variables { first, after };
            let data = client.execute::<RolesList>(vars).await?;
            let roles = data.roles;
            let nodes = roles.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|r| {
                        let permission_sets = r
                            .permission_sets
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        vec![r.role_id.clone(), r.name.clone(), permission_sets]
                    })
                    .collect();
                output::print_table(&["Role ID", "Name", "Permission Sets"], rows);
                let pi = roles.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        RoleAction::Get { id } => {
            let vars = role_get::Variables { id };
            let data = client.execute::<RoleGet>(vars).await?;
            match data.role {
                Some(role) => {
                    if json {
                        output::print_json(&role)?;
                    } else {
                        let permission_sets = role
                            .permission_sets
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        output::print_kv(&[
                            ("Role ID", &role.role_id),
                            ("Name", &role.name),
                            ("Created At", &role.created_at),
                            ("Permission Sets", &permission_sets),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Role not found");
                    }
                }
            }
        }
        RoleAction::AddPermissionSets {
            role_id,
            permission_set_ids,
        } => {
            let vars = role_add_permission_sets::Variables {
                input: role_add_permission_sets::RoleAddPermissionSetsInput {
                    role_id,
                    permission_set_ids,
                },
            };
            let data = client.execute::<RoleAddPermissionSets>(vars).await?;
            let role = data.role_add_permission_sets.role;
            if json {
                output::print_json(&role)?;
            } else {
                let permission_sets = role
                    .permission_sets
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                output::print_kv(&[
                    ("Role ID", &role.role_id),
                    ("Name", &role.name),
                    ("Created At", &role.created_at),
                    ("Permission Sets", &permission_sets),
                ]);
            }
        }
        RoleAction::RemovePermissionSets {
            role_id,
            permission_set_ids,
        } => {
            let vars = role_remove_permission_sets::Variables {
                input: role_remove_permission_sets::RoleRemovePermissionSetsInput {
                    role_id,
                    permission_set_ids,
                },
            };
            let data = client.execute::<RoleRemovePermissionSets>(vars).await?;
            let role = data.role_remove_permission_sets.role;
            if json {
                output::print_json(&role)?;
            } else {
                let permission_sets = role
                    .permission_sets
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                output::print_kv(&[
                    ("Role ID", &role.role_id),
                    ("Name", &role.name),
                    ("Created At", &role.created_at),
                    ("Permission Sets", &permission_sets),
                ]);
            }
        }
    }
    Ok(())
}
