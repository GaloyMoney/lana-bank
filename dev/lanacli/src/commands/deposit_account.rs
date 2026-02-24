use anyhow::Result;

use crate::cli::DepositAccountAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar, sval};

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
        DepositAccountAction::RecordDeposit {
            deposit_account_id,
            amount,
        } => {
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
                output::print_json(&d)?;
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
        }
        DepositAccountAction::InitiateWithdrawal {
            deposit_account_id,
            amount,
            reference,
        } => {
            let vars = withdrawal_initiate::Variables {
                input: withdrawal_initiate::WithdrawalInitiateInput {
                    deposit_account_id,
                    amount: sval(amount),
                    reference: Some(reference),
                },
            };
            let data = client.execute::<WithdrawalInitiate>(vars).await?;
            let w = data.withdrawal_initiate.withdrawal;
            if json {
                output::print_json(&w)?;
            } else {
                let settled = scalar(&w.account.balance.settled);
                let pending = scalar(&w.account.balance.pending);
                output::print_kv(&[
                    ("Withdrawal ID", &w.withdrawal_id),
                    ("Approval Process ID", &w.approval_process_id),
                    ("Status", &format!("{:?}", w.status)),
                    ("Account Status", &format!("{:?}", w.account.status)),
                    ("Balance (settled)", &settled),
                    ("Balance (pending)", &pending),
                ]);
            }
        }
        DepositAccountAction::ConfirmWithdrawal { withdrawal_id } => {
            let vars = withdrawal_confirm::Variables {
                input: withdrawal_confirm::WithdrawalConfirmInput { withdrawal_id },
            };
            let data = client.execute::<WithdrawalConfirm>(vars).await?;
            let w = data.withdrawal_confirm.withdrawal;
            if json {
                output::print_json(&w)?;
            } else {
                let settled = scalar(&w.account.balance.settled);
                let pending = scalar(&w.account.balance.pending);
                output::print_kv(&[
                    ("Withdrawal ID", &w.withdrawal_id),
                    ("Status", &format!("{:?}", w.status)),
                    ("Balance (settled)", &settled),
                    ("Balance (pending)", &pending),
                ]);
            }
        }
        DepositAccountAction::CancelWithdrawal { withdrawal_id } => {
            let vars = withdrawal_cancel::Variables {
                input: withdrawal_cancel::WithdrawalCancelInput { withdrawal_id },
            };
            let data = client.execute::<WithdrawalCancel>(vars).await?;
            let w = data.withdrawal_cancel.withdrawal;
            if json {
                output::print_json(&w)?;
            } else {
                let settled = scalar(&w.account.balance.settled);
                let pending = scalar(&w.account.balance.pending);
                output::print_kv(&[
                    ("Withdrawal ID", &w.withdrawal_id),
                    ("Status", &format!("{:?}", w.status)),
                    ("Balance (settled)", &settled),
                    ("Balance (pending)", &pending),
                ]);
            }
        }
        DepositAccountAction::RevertWithdrawal { withdrawal_id } => {
            let vars = withdrawal_revert::Variables {
                input: withdrawal_revert::WithdrawalRevertInput { withdrawal_id },
            };
            let data = client.execute::<WithdrawalRevert>(vars).await?;
            let w = data.withdrawal_revert.withdrawal;
            if json {
                output::print_json(&w)?;
            } else {
                output::print_kv(&[
                    ("Withdrawal ID", &w.withdrawal_id),
                    ("Status", &format!("{:?}", w.status)),
                ]);
            }
        }
    }
    Ok(())
}
