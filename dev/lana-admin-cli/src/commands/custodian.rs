use anyhow::Result;

use crate::cli::CustodianAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: CustodianAction,
    json: bool,
) -> Result<()> {
    match action {
        CustodianAction::Create { input_json } => {
            let input: custodian_create::CustodianCreateInput = serde_json::from_str(&input_json)?;
            let vars = custodian_create::Variables { input };
            let data = client.execute::<CustodianCreate>(vars).await?;
            let c = data.custodian_create.custodian;
            if json {
                output::print_json(&c)?;
            } else {
                output::print_kv(&[("Custodian ID", &c.custodian_id)]);
            }
        }
        CustodianAction::ConfigUpdate {
            custodian_id,
            config_json,
        } => {
            let config: custodian_config_update::CustodianConfigInput =
                serde_json::from_str(&config_json)?;
            let vars = custodian_config_update::Variables {
                input: custodian_config_update::CustodianConfigUpdateInput {
                    custodian_id,
                    config,
                },
            };
            let data = client.execute::<CustodianConfigUpdate>(vars).await?;
            let c = data.custodian_config_update.custodian;
            if json {
                output::print_json(&c)?;
            } else {
                output::print_kv(&[("Custodian ID", &c.custodian_id)]);
            }
        }
    }
    Ok(())
}
