use anyhow::Result;

use crate::cli::CustomerAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: CustomerAction, json: bool) -> Result<()> {
    match action {
        CustomerAction::List { first, after } => {
            let vars = customers_list::Variables { first, after };
            let data = client.execute::<CustomersList>(vars).await?;
            let nodes = data.customers.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|c| {
                        vec![
                            c.customer_id.clone(),
                            c.public_id.clone(),
                            c.email.clone(),
                            format!("{:?}", c.customer_type),
                            format!("{:?}", c.activity),
                            format!("{:?}", c.level),
                            c.created_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &[
                        "ID",
                        "Public ID",
                        "Email",
                        "Type",
                        "Activity",
                        "Level",
                        "Created",
                    ],
                    rows,
                );
                let pi = data.customers.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        CustomerAction::Get { id } => {
            let vars = customer_get::Variables { id };
            let data = client.execute::<CustomerGet>(vars).await?;
            match data.customer {
                Some(c) => {
                    if json {
                        output::print_json(&c)?;
                    } else {
                        output::print_kv(&[
                            ("Customer ID", &c.customer_id),
                            ("Public ID", &c.public_id),
                            ("Email", &c.email),
                            ("Telegram", &c.telegram_handle),
                            ("Type", &format!("{:?}", c.customer_type)),
                            ("Activity", &format!("{:?}", c.activity)),
                            ("Level", &format!("{:?}", c.level)),
                            ("KYC", &format!("{:?}", c.kyc_verification)),
                            ("Created", &c.created_at),
                            ("Applicant ID", &c.applicant_id),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Customer not found");
                    }
                }
            }
        }
        CustomerAction::GetByEmail { email } => {
            let vars = customer_get_by_email::Variables { email };
            let data = client.execute::<CustomerGetByEmail>(vars).await?;
            match data.customer_by_email {
                Some(c) => {
                    if json {
                        output::print_json(&c)?;
                    } else {
                        output::print_kv(&[
                            ("Customer ID", &c.customer_id),
                            ("Public ID", &c.public_id),
                            ("Email", &c.email),
                            ("Telegram", &c.telegram_handle),
                            ("Type", &format!("{:?}", c.customer_type)),
                            ("Activity", &format!("{:?}", c.activity)),
                            ("Level", &format!("{:?}", c.level)),
                            ("KYC", &format!("{:?}", c.kyc_verification)),
                            ("Created", &c.created_at),
                            ("Applicant ID", &c.applicant_id),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Customer not found");
                    }
                }
            }
        }
    }
    Ok(())
}
