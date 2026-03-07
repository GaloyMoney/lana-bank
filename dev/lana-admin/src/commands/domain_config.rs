use anyhow::Result;

use crate::cli::DomainConfigAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: DomainConfigAction,
    json: bool,
) -> Result<()> {
    match action {
        DomainConfigAction::List => {
            let vars = domain_configs_list::Variables {};
            let data = client.execute::<DomainConfigsList>(vars).await?;
            let edges = data.domain_configs.edges;
            if json {
                output::print_json(&edges)?;
            } else {
                let rows: Vec<Vec<String>> = edges
                    .iter()
                    .map(|e| {
                        vec![
                            e.node.domain_config_id.clone(),
                            e.node.key.clone(),
                            format!("{:?}", e.node.config_type),
                            e.node.encrypted.to_string(),
                            e.node.is_set.to_string(),
                            e.node.value.to_string(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &["Config ID", "Key", "Type", "Encrypted", "Is Set", "Value"],
                    rows,
                );
            }
        }
        DomainConfigAction::Update {
            domain_config_id,
            value_json,
        } => {
            let value: serde_json::Value = serde_json::from_str(&value_json)?;
            let vars = domain_config_update::Variables {
                input: domain_config_update::DomainConfigUpdateInput {
                    domain_config_id,
                    value,
                },
            };
            let data = client.execute::<DomainConfigUpdate>(vars).await?;
            let dc = data.domain_config_update.domain_config;
            if json {
                output::print_json(&dc)?;
            } else {
                output::print_kv(&[
                    ("Config ID", &dc.domain_config_id),
                    ("Key", &dc.key),
                    ("Type", &format!("{:?}", dc.config_type)),
                    ("Encrypted", &dc.encrypted.to_string()),
                    ("Is Set", &dc.is_set.to_string()),
                    ("Value", &dc.value.to_string()),
                ]);
            }
        }
    }
    Ok(())
}
