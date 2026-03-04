use anyhow::Result;

use crate::cli::WithdrawalAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: WithdrawalAction,
    json: bool,
) -> Result<()> {
    match action {
        WithdrawalAction::Find { id } => {
            let vars = withdrawal_find::Variables { id };
            let data = client.execute::<WithdrawalFind>(vars).await?;
            match data.withdrawal {
                Some(w) => {
                    if json {
                        output::print_json(&w)?;
                    } else {
                        output::print_kv(&[
                            ("Withdrawal ID", &w.withdrawal_id),
                            ("Status", &format!("{:?}", w.status)),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Withdrawal not found");
                    }
                }
            }
        }
    }
    Ok(())
}
