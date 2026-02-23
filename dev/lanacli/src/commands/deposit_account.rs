use anyhow::Result;

use crate::cli::DepositAccountAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar};

pub async fn execute(
    client: &mut GraphQLClient,
    action: DepositAccountAction,
    json: bool,
) -> Result<()> {
    match action {
        DepositAccountAction::Create { customer_id } => {
            let vars = deposit_account_create::Variables {
                input: deposit_account_create::DepositAccountCreateInput {
                    customer_id: customer_id,
                },
            };
            let data = client.execute::<DepositAccountCreate>(vars).await?;
            let a = data.deposit_account_create.account;
            if json {
                output::print_json(&a)?;
            } else {
                output::print_kv(&[
                    ("Account ID", &a.deposit_account_id),
                    ("Customer ID", &a.customer_id),
                    ("Public ID", &a.public_id),
                    ("Status", &format!("{:?}", a.status)),
                    ("Created", &a.created_at),
                ]);
            }
        }
        DepositAccountAction::List { first, after } => {
            let vars = deposit_accounts_list::Variables { first, after };
            let data = client.execute::<DepositAccountsList>(vars).await?;
            let nodes = data.deposit_accounts.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|a| {
                        vec![
                            a.deposit_account_id.clone(),
                            a.customer_id.clone(),
                            a.public_id.clone(),
                            format!("{:?}", a.status),
                            a.created_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &[
                        "Account ID",
                        "Customer ID",
                        "Public ID",
                        "Status",
                        "Created",
                    ],
                    rows,
                );
                let pi = data.deposit_accounts.page_info;
                if pi.has_next_page {
                    if let Some(cursor) = pi.end_cursor {
                        println!("\nMore results available. Use --after {cursor}");
                    }
                }
            }
        }
        DepositAccountAction::Get { id } => {
            let vars = deposit_account_get::Variables { id };
            let data = client.execute::<DepositAccountGet>(vars).await?;
            match data.deposit_account {
                Some(a) => {
                    if json {
                        output::print_json(&a)?;
                    } else {
                        let settled = scalar(&a.balance.settled);
                        let pending = scalar(&a.balance.pending);
                        output::print_kv(&[
                            ("Account ID", &a.deposit_account_id),
                            ("Customer ID", &a.customer_id),
                            ("Public ID", &a.public_id),
                            ("Status", &format!("{:?}", a.status)),
                            ("Created", &a.created_at),
                            ("Balance (settled)", &settled),
                            ("Balance (pending)", &pending),
                        ]);
                    }
                }
                None => println!("Deposit account not found"),
            }
        }
    }
    Ok(())
}
