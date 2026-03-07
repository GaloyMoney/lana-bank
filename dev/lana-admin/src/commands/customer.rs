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
                        let (first_name, last_name, company_name) = personal_info_values(
                            c.personal_info
                                .as_ref()
                                .map(|info| info.first_name.as_str()),
                            c.personal_info.as_ref().map(|info| info.last_name.as_str()),
                            c.personal_info
                                .as_ref()
                                .and_then(|info| info.company_name.as_deref()),
                        );
                        vec![
                            c.customer_id.clone(),
                            c.public_id.clone(),
                            c.email.clone(),
                            format!("{:?}", c.customer_type),
                            display_name(&first_name, &last_name, &company_name),
                            format!("{:?}", c.status),
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
                        "Name",
                        "Status",
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
                        let (first_name, last_name, company_name) = personal_info_values(
                            c.personal_info
                                .as_ref()
                                .map(|info| info.first_name.as_str()),
                            c.personal_info.as_ref().map(|info| info.last_name.as_str()),
                            c.personal_info
                                .as_ref()
                                .and_then(|info| info.company_name.as_deref()),
                        );
                        let first_name = display_value(&first_name);
                        let last_name = display_value(&last_name);
                        let company_name = display_value(&company_name);
                        output::print_kv(&[
                            ("Customer ID", &c.customer_id),
                            ("Public ID", &c.public_id),
                            ("Email", &c.email),
                            ("Telegram", &c.telegram_handle),
                            ("Type", &format!("{:?}", c.customer_type)),
                            ("First Name", &first_name),
                            ("Last Name", &last_name),
                            ("Company Name", &company_name),
                            ("Status", &format!("{:?}", c.status)),
                            ("Level", &format!("{:?}", c.level)),
                            ("KYC", &format!("{:?}", c.kyc_verification)),
                            ("Created", &c.created_at),
                            ("Applicant ID", &c.applicant_id),
                        ]);
                    }
                }
                None => output::not_found("Customer", json),
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
                        let (first_name, last_name, company_name) = personal_info_values(
                            c.personal_info
                                .as_ref()
                                .map(|info| info.first_name.as_str()),
                            c.personal_info.as_ref().map(|info| info.last_name.as_str()),
                            c.personal_info
                                .as_ref()
                                .and_then(|info| info.company_name.as_deref()),
                        );
                        let first_name = display_value(&first_name);
                        let last_name = display_value(&last_name);
                        let company_name = display_value(&company_name);
                        output::print_kv(&[
                            ("Customer ID", &c.customer_id),
                            ("Public ID", &c.public_id),
                            ("Email", &c.email),
                            ("Telegram", &c.telegram_handle),
                            ("Type", &format!("{:?}", c.customer_type)),
                            ("First Name", &first_name),
                            ("Last Name", &last_name),
                            ("Company Name", &company_name),
                            ("Status", &format!("{:?}", c.status)),
                            ("Level", &format!("{:?}", c.level)),
                            ("KYC", &format!("{:?}", c.kyc_verification)),
                            ("Created", &c.created_at),
                            ("Applicant ID", &c.applicant_id),
                        ]);
                    }
                }
                None => output::not_found("Customer", json),
            }
        }
        CustomerAction::Close { customer_id } => {
            let vars = customer_close::Variables {
                input: customer_close::CustomerCloseInput { customer_id },
            };
            let data = client.execute::<CustomerClose>(vars).await?;
            let c = data.customer_close.customer;
            if json {
                output::print_json(&c)?;
            } else {
                let (first_name, last_name, company_name) = personal_info_values(
                    c.personal_info
                        .as_ref()
                        .map(|info| info.first_name.as_str()),
                    c.personal_info.as_ref().map(|info| info.last_name.as_str()),
                    c.personal_info
                        .as_ref()
                        .and_then(|info| info.company_name.as_deref()),
                );
                let first_name = display_value(&first_name);
                let last_name = display_value(&last_name);
                let company_name = display_value(&company_name);
                output::print_kv(&[
                    ("Customer ID", &c.customer_id),
                    ("Public ID", &c.public_id),
                    ("Email", &c.email),
                    ("First Name", &first_name),
                    ("Last Name", &last_name),
                    ("Company Name", &company_name),
                    ("Status", &format!("{:?}", c.status)),
                    ("Created", &c.created_at),
                ]);
                println!("\nCustomer closed.");
            }
        }
    }
    Ok(())
}

fn personal_info_values<'a>(
    first_name: Option<&'a str>,
    last_name: Option<&'a str>,
    company_name: Option<&'a str>,
) -> (String, String, String) {
    (
        first_name.unwrap_or_default().to_string(),
        last_name.unwrap_or_default().to_string(),
        company_name.unwrap_or_default().to_string(),
    )
}

fn display_name(first_name: &str, last_name: &str, company_name: &str) -> String {
    if !company_name.is_empty() {
        company_name.to_string()
    } else {
        let full_name = [first_name, last_name]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if full_name.is_empty() {
            "N/A".to_string()
        } else {
            full_name
        }
    }
}

fn display_value(value: &str) -> String {
    if value.is_empty() {
        "N/A".to_string()
    } else {
        value.to_string()
    }
}
