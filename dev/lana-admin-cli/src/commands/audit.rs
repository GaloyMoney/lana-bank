use anyhow::Result;

use crate::cli::AuditAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: AuditAction, json: bool) -> Result<()> {
    match action {
        AuditAction::List { first, after } => {
            let vars = audit_logs_list::Variables { first, after };
            let data = client.execute::<AuditLogsList>(vars).await?;
            let edges = data.audit.edges;
            if json {
                output::print_json(&edges)?;
            } else {
                let rows: Vec<Vec<String>> = edges
                    .iter()
                    .map(|e| {
                        let n = &e.node;
                        let subject = format_audit_subject_list(&n.subject);
                        vec![
                            n.id.clone(),
                            subject,
                            n.object.clone(),
                            n.action.clone(),
                            n.authorized.to_string(),
                            n.recorded_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &[
                        "ID",
                        "Subject",
                        "Object",
                        "Action",
                        "Authorized",
                        "Recorded At",
                    ],
                    rows,
                );
                let pi = data.audit.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        AuditAction::Customer { id } => {
            let vars = customer_audit_log::Variables { id };
            let data = client.execute::<CustomerAuditLog>(vars).await?;
            let nodes = data.audit.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|n| {
                        let subject = format_audit_subject_customer(&n.subject);
                        vec![
                            subject,
                            n.object.clone(),
                            n.action.clone(),
                            n.authorized.to_string(),
                            n.recorded_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &["Subject", "Object", "Action", "Authorized", "Recorded At"],
                    rows,
                );
            }
        }
    }
    Ok(())
}

fn format_audit_subject_list(
    subject: &audit_logs_list::AuditLogsListAuditEdgesNodeSubject,
) -> String {
    match subject {
        audit_logs_list::AuditLogsListAuditEdgesNodeSubject::User(u) => u.email.clone(),
        audit_logs_list::AuditLogsListAuditEdgesNodeSubject::System(s) => {
            format!("System:{}", s.actor)
        }
    }
}

fn format_audit_subject_customer(
    subject: &customer_audit_log::CustomerAuditLogAuditNodesSubject,
) -> String {
    match subject {
        customer_audit_log::CustomerAuditLogAuditNodesSubject::User(u) => u.email.clone(),
        customer_audit_log::CustomerAuditLogAuditNodesSubject::System(s) => {
            format!("System:{}", s.actor)
        }
    }
}
