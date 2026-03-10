use anyhow::Result;
use serde_json::json;

use crate::cli::DepositAccountAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar, sval};

pub async fn record_deposit(
    client: &mut GraphQLClient,
    deposit_account_id: String,
    amount: String,
    json: bool,
) -> Result<()> {
    let vars = deposit_record::Variables {
        input: deposit_record::DepositRecordInput {
            deposit_account_id,
            amount: sval(amount),
            reference: None,
        },
    };
    let data = client.execute::<DepositRecord>(vars).await?;
    let d = data.deposit_record.deposit;
    if json {
        output::print_json(&json!({
            "depositId": d.deposit_id,
            "depositAccountId": d.account.deposit_account_id,
            "amount": d.amount,
            "settledBalance": d.account.balance.settled,
            "pendingBalance": d.account.balance.pending,
        }))?;
    } else {
        let amount = scalar(&d.amount);
        let settled = scalar(&d.account.balance.settled);
        let pending = scalar(&d.account.balance.pending);
        output::print_kv(&[
            ("Deposit ID", &d.deposit_id),
            ("Amount", &amount),
            ("Balance (settled)", &settled),
            ("Balance (pending)", &pending),
        ]);
    }
    Ok(())
}

pub async fn execute(
    client: &mut GraphQLClient,
    action: DepositAccountAction,
    json: bool,
) -> Result<()> {
    match action {
        DepositAccountAction::Create { customer_id } => {
            let vars = deposit_account_create::Variables {
                input: deposit_account_create::DepositAccountCreateInput { customer_id },
            };
            let data = client.execute::<DepositAccountCreate>(vars).await?;
            let a = data.deposit_account_create.account;
            if json {
                output::print_json(&json!({
                    "depositAccountId": a.deposit_account_id,
                    "customerId": a.customer_id,
                    "publicId": a.public_id,
                    "status": format!("{:?}", a.status),
                    "createdAt": a.created_at,
                }))?;
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
                            format!("{:?}", a.activity),
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
                        "Activity",
                        "Created",
                    ],
                    rows,
                );
                let pi = data.deposit_accounts.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        DepositAccountAction::Get { id } => {
            let vars = deposit_account_get::Variables { id };
            let data = client.execute::<DepositAccountGet>(vars).await?;
            match data.deposit_account {
                Some(a) => {
                    if json {
                        output::print_json(&json!({
                            "depositAccountId": a.deposit_account_id,
                            "customerId": a.customer_id,
                            "publicId": a.public_id,
                            "status": format!("{:?}", a.status),
                            "activity": format!("{:?}", a.activity),
                            "createdAt": a.created_at,
                            "settledBalance": a.balance.settled,
                            "pendingBalance": a.balance.pending,
                        }))?;
                    } else {
                        let settled = scalar(&a.balance.settled);
                        let pending = scalar(&a.balance.pending);
                        output::print_kv(&[
                            ("Account ID", &a.deposit_account_id),
                            ("Customer ID", &a.customer_id),
                            ("Public ID", &a.public_id),
                            ("Status", &format!("{:?}", a.status)),
                            ("Activity", &format!("{:?}", a.activity)),
                            ("Created", &a.created_at),
                            ("Balance (settled)", &settled),
                            ("Balance (pending)", &pending),
                        ]);
                    }
                }
                None => output::not_found("Deposit account", json),
            }
        }
        DepositAccountAction::Freeze { deposit_account_id } => {
            let vars = deposit_account_freeze::Variables {
                input: deposit_account_freeze::DepositAccountFreezeInput { deposit_account_id },
            };
            let data = client.execute::<DepositAccountFreeze>(vars).await?;
            let a = data.deposit_account_freeze.account;
            if json {
                output::print_json(&json!({
                    "depositAccountId": a.deposit_account_id,
                    "status": format!("{:?}", a.status),
                    "settledBalance": a.balance.settled,
                }))?;
            } else {
                let settled = scalar(&a.balance.settled);
                output::print_kv(&[
                    ("Account ID", &a.deposit_account_id),
                    ("Status", &format!("{:?}", a.status)),
                    ("Balance (settled)", &settled),
                ]);
            }
        }
        DepositAccountAction::Unfreeze { deposit_account_id } => {
            let vars = deposit_account_unfreeze::Variables {
                input: deposit_account_unfreeze::DepositAccountUnfreezeInput { deposit_account_id },
            };
            let data = client.execute::<DepositAccountUnfreeze>(vars).await?;
            let a = data.deposit_account_unfreeze.account;
            if json {
                output::print_json(&json!({
                    "depositAccountId": a.deposit_account_id,
                    "status": format!("{:?}", a.status),
                    "settledBalance": a.balance.settled,
                }))?;
            } else {
                let settled = scalar(&a.balance.settled);
                output::print_kv(&[
                    ("Account ID", &a.deposit_account_id),
                    ("Status", &format!("{:?}", a.status)),
                    ("Balance (settled)", &settled),
                ]);
            }
        }
        DepositAccountAction::Close { deposit_account_id } => {
            let vars = deposit_account_close::Variables {
                input: deposit_account_close::DepositAccountCloseInput { deposit_account_id },
            };
            let data = client.execute::<DepositAccountClose>(vars).await?;
            let a = data.deposit_account_close.account;
            if json {
                output::print_json(&json!({
                    "depositAccountId": a.deposit_account_id,
                    "status": format!("{:?}", a.status),
                    "settledBalance": a.balance.settled,
                }))?;
            } else {
                let settled = scalar(&a.balance.settled);
                output::print_kv(&[
                    ("Account ID", &a.deposit_account_id),
                    ("Status", &format!("{:?}", a.status)),
                    ("Balance (settled)", &settled),
                ]);
            }
        }
    }
    Ok(())
}
