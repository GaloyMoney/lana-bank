use anyhow::Result;
use serde_json::json;

use crate::cli::WithdrawalAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar, sval};

pub async fn execute(
    client: &mut GraphQLClient,
    action: WithdrawalAction,
    json: bool,
) -> Result<()> {
    match action {
        WithdrawalAction::Initiate {
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
                output::print_json(&json!({
                    "withdrawalId": w.withdrawal_id,
                    "approvalProcessId": w.approval_process_id,
                    "accountId": w.account_id,
                    "amount": w.amount,
                    "status": format!("{:?}", w.status),
                    "accountStatus": format!("{:?}", w.account.status),
                    "createdAt": w.created_at,
                    "publicId": w.public_id,
                    "reference": w.reference,
                    "settledBalance": w.account.balance.settled,
                    "pendingBalance": w.account.balance.pending,
                }))?;
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
        WithdrawalAction::Confirm { withdrawal_id } => {
            let vars = withdrawal_confirm::Variables {
                input: withdrawal_confirm::WithdrawalConfirmInput { withdrawal_id },
            };
            let data = client.execute::<WithdrawalConfirm>(vars).await?;
            let w = data.withdrawal_confirm.withdrawal;
            if json {
                output::print_json(&json!({
                    "withdrawalId": w.withdrawal_id,
                    "accountId": w.account_id,
                    "amount": w.amount,
                    "status": format!("{:?}", w.status),
                    "createdAt": w.created_at,
                    "publicId": w.public_id,
                    "reference": w.reference,
                    "settledBalance": w.account.balance.settled,
                    "pendingBalance": w.account.balance.pending,
                }))?;
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
        WithdrawalAction::Cancel { withdrawal_id } => {
            let vars = withdrawal_cancel::Variables {
                input: withdrawal_cancel::WithdrawalCancelInput { withdrawal_id },
            };
            let data = client.execute::<WithdrawalCancel>(vars).await?;
            let w = data.withdrawal_cancel.withdrawal;
            if json {
                output::print_json(&json!({
                    "withdrawalId": w.withdrawal_id,
                    "accountId": w.account_id,
                    "amount": w.amount,
                    "status": format!("{:?}", w.status),
                    "createdAt": w.created_at,
                    "publicId": w.public_id,
                    "reference": w.reference,
                    "settledBalance": w.account.balance.settled,
                    "pendingBalance": w.account.balance.pending,
                }))?;
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
        WithdrawalAction::Revert { withdrawal_id } => {
            let vars = withdrawal_revert::Variables {
                input: withdrawal_revert::WithdrawalRevertInput { withdrawal_id },
            };
            let data = client.execute::<WithdrawalRevert>(vars).await?;
            let w = data.withdrawal_revert.withdrawal;
            if json {
                output::print_json(&json!({
                    "withdrawalId": w.withdrawal_id,
                    "accountId": w.account_id,
                    "amount": w.amount,
                    "status": format!("{:?}", w.status),
                    "createdAt": w.created_at,
                    "publicId": w.public_id,
                    "reference": w.reference,
                }))?;
            } else {
                output::print_kv(&[
                    ("Withdrawal ID", &w.withdrawal_id),
                    ("Status", &format!("{:?}", w.status)),
                ]);
            }
        }
        WithdrawalAction::Find { id } => {
            let vars = withdrawal_find::Variables { id };
            let data = client.execute::<WithdrawalFind>(vars).await?;
            match data.withdrawal {
                Some(w) => {
                    if json {
                        output::print_json(&json!({
                            "withdrawalId": w.withdrawal_id,
                            "approvalProcessId": w.approval_process_id,
                            "accountId": w.account_id,
                            "amount": w.amount,
                            "status": format!("{:?}", w.status),
                            "createdAt": w.created_at,
                            "publicId": w.public_id,
                            "reference": w.reference,
                        }))?;
                    } else {
                        output::print_kv(&[
                            ("Withdrawal ID", &w.withdrawal_id),
                            ("Approval Process ID", &w.approval_process_id),
                            ("Status", &format!("{:?}", w.status)),
                        ]);
                    }
                }
                None => output::not_found("Withdrawal", json),
            }
        }
    }
    Ok(())
}
