use anyhow::Result;

use crate::cli::CollateralAction;
use crate::client::GraphQLClient;
use crate::date::normalize_graphql_date;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: CollateralAction,
    json: bool,
) -> Result<()> {
    match action {
        CollateralAction::Update {
            collateral_id,
            collateral,
            effective,
        } => {
            let effective = normalize_graphql_date(&effective)?;
            let vars = collateral_update::Variables {
                input: collateral_update::CollateralUpdateInput {
                    collateral_id,
                    collateral: serde_json::Value::Number(serde_json::Number::from(collateral)),
                    effective,
                },
            };
            let data = client.execute::<CollateralUpdate>(vars).await?;
            let c = data.collateral_update.collateral;
            if json {
                output::print_json(&c)?;
            } else {
                output::print_kv(&[("Collateral ID", &c.collateral_id)]);
            }
        }
    }
    Ok(())
}
