use anyhow::Result;

use crate::cli::ApprovalProcessAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: ApprovalProcessAction,
    json: bool,
) -> Result<()> {
    match action {
        ApprovalProcessAction::Approve { process_id } => {
            let vars = approval_process_approve::Variables {
                input: approval_process_approve::ApprovalProcessApproveInput { process_id },
            };
            let data = client.execute::<ApprovalProcessApprove>(vars).await?;
            let ap = data.approval_process_approve.approval_process;
            if json {
                output::print_json(&ap)?;
            } else {
                output::print_kv(&[
                    ("Process ID", &ap.approval_process_id),
                    ("Type", &format!("{:?}", ap.approval_process_type)),
                    ("Status", &format!("{:?}", ap.status)),
                    ("Created", &ap.created_at),
                ]);
                println!("\nApproval submitted.");
            }
        }
        ApprovalProcessAction::Deny { process_id, reason } => {
            let vars = approval_process_deny::Variables {
                input: approval_process_deny::ApprovalProcessDenyInput { process_id },
                reason,
            };
            let data = client.execute::<ApprovalProcessDeny>(vars).await?;
            let ap = data.approval_process_deny.approval_process;
            if json {
                output::print_json(&ap)?;
            } else {
                output::print_kv(&[
                    ("Process ID", &ap.approval_process_id),
                    ("Type", &format!("{:?}", ap.approval_process_type)),
                    ("Status", &format!("{:?}", ap.status)),
                    (
                        "Denied Reason",
                        ap.denied_reason.as_deref().unwrap_or("N/A"),
                    ),
                    ("Created", &ap.created_at),
                ]);
                println!("\nDenial submitted.");
            }
        }
        ApprovalProcessAction::List { first, after } => {
            let vars = approval_processes_list::Variables { first, after };
            let data = client.execute::<ApprovalProcessesList>(vars).await?;
            let nodes = data.approval_processes.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|ap| {
                        vec![
                            ap.approval_process_id.clone(),
                            format!("{:?}", ap.approval_process_type),
                            format!("{:?}", ap.status),
                            ap.created_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(&["Process ID", "Type", "Status", "Created"], rows);
                let pi = data.approval_processes.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        ApprovalProcessAction::Get { id } => {
            let vars = approval_process_get::Variables { id };
            let data = client.execute::<ApprovalProcessGet>(vars).await?;
            match data.approval_process {
                Some(ap) => {
                    if json {
                        output::print_json(&ap)?;
                    } else {
                        output::print_kv(&[
                            ("Process ID", &ap.approval_process_id),
                            ("Type", &format!("{:?}", ap.approval_process_type)),
                            ("Status", &format!("{:?}", ap.status)),
                            (
                                "Denied Reason",
                                ap.denied_reason.as_deref().unwrap_or("N/A"),
                            ),
                            (
                                "Can Submit Decision",
                                &ap.user_can_submit_decision.to_string(),
                            ),
                            ("Created", &ap.created_at),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Approval process not found");
                    }
                }
            }
        }
    }
    Ok(())
}
