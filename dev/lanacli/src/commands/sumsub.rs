use anyhow::Result;

use crate::cli::SumsubAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: SumsubAction, json: bool) -> Result<()> {
    match action {
        SumsubAction::PermalinkCreate { prospect_id } => {
            let vars = sumsub_permalink_create::Variables {
                input: sumsub_permalink_create::SumsubPermalinkCreateInput { prospect_id },
            };
            let data = client.execute::<SumsubPermalinkCreate>(vars).await?;
            let result = data.sumsub_permalink_create;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[("URL", &result.url)]);
            }
        }
    }
    Ok(())
}
