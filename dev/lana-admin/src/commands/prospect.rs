use anyhow::Result;

use crate::cli::ProspectAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(client: &mut GraphQLClient, action: ProspectAction, json: bool) -> Result<()> {
    match action {
        ProspectAction::Create {
            email,
            telegram_handle,
            customer_type,
        } => {
            let ct = parse_customer_type(&customer_type)?;
            let vars = prospect_create::Variables {
                input: prospect_create::ProspectCreateInput {
                    email,
                    telegram_handle,
                    customer_type: ct,
                },
            };
            let data = client.execute::<ProspectCreate>(vars).await?;
            let p = data.prospect_create.prospect;
            if json {
                output::print_json(&p)?;
            } else {
                output::print_kv(&[
                    ("Prospect ID", &p.prospect_id),
                    ("Public ID", &p.public_id),
                    ("Email", &p.email),
                    ("Telegram", &p.telegram_handle),
                    ("Type", &format!("{:?}", p.customer_type)),
                    ("Stage", &format!("{:?}", p.stage)),
                    ("Created", &p.created_at),
                ]);
            }
        }
        ProspectAction::List { first, after } => {
            let vars = prospects_list::Variables { first, after };
            let data = client.execute::<ProspectsList>(vars).await?;
            let nodes = data.prospects.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|p| {
                        let (first_name, last_name, company_name) = personal_info_values(
                            p.personal_info
                                .as_ref()
                                .map(|info| info.first_name.as_str()),
                            p.personal_info.as_ref().map(|info| info.last_name.as_str()),
                            p.personal_info
                                .as_ref()
                                .and_then(|info| info.company_name.as_deref()),
                        );
                        vec![
                            p.prospect_id.clone(),
                            p.public_id.clone(),
                            p.email.clone(),
                            format!("{:?}", p.customer_type),
                            display_name(&first_name, &last_name, &company_name),
                            format!("{:?}", p.stage),
                            p.created_at.clone(),
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
                        "Stage",
                        "Created",
                    ],
                    rows,
                );
                let pi = data.prospects.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        ProspectAction::Get { id } => {
            let vars = prospect_get::Variables { id };
            let data = client.execute::<ProspectGet>(vars).await?;
            match data.prospect {
                Some(p) => {
                    if json {
                        output::print_json(&p)?;
                    } else {
                        let (first_name, last_name, company_name) = personal_info_values(
                            p.personal_info
                                .as_ref()
                                .map(|info| info.first_name.as_str()),
                            p.personal_info.as_ref().map(|info| info.last_name.as_str()),
                            p.personal_info
                                .as_ref()
                                .and_then(|info| info.company_name.as_deref()),
                        );
                        let first_name = display_value(&first_name);
                        let last_name = display_value(&last_name);
                        let company_name = display_value(&company_name);
                        output::print_kv(&[
                            ("Prospect ID", &p.prospect_id),
                            ("Public ID", &p.public_id),
                            ("Email", &p.email),
                            ("Telegram", &p.telegram_handle),
                            ("Type", &format!("{:?}", p.customer_type)),
                            ("First Name", &first_name),
                            ("Last Name", &last_name),
                            ("Company Name", &company_name),
                            ("Stage", &format!("{:?}", p.stage)),
                            ("KYC Status", &format!("{:?}", p.kyc_status)),
                            ("Level", &format!("{:?}", p.level)),
                            ("Created", &p.created_at),
                            ("Applicant ID", p.applicant_id.as_deref().unwrap_or("N/A")),
                        ]);
                    }
                }
                None => output::not_found("Prospect", json),
            }
        }
        ProspectAction::Convert { prospect_id } => {
            let vars = prospect_convert::Variables {
                input: prospect_convert::ProspectConvertInput { prospect_id },
            };
            let data = client.execute::<ProspectConvert>(vars).await?;
            let c = data.prospect_convert.customer;
            if json {
                output::print_json(&c)?;
            } else {
                output::print_kv(&[
                    ("Customer ID", &c.customer_id),
                    ("Public ID", &c.public_id),
                    ("Email", &c.email),
                    ("Type", &format!("{:?}", c.customer_type)),
                    ("Created", &c.created_at),
                ]);
                println!("\nProspect converted to customer successfully.");
            }
        }
        ProspectAction::Close { prospect_id } => {
            let vars = prospect_close::Variables {
                input: prospect_close::ProspectCloseInput { prospect_id },
            };
            let data = client.execute::<ProspectClose>(vars).await?;
            let p = data.prospect_close.prospect;
            if json {
                output::print_json(&p)?;
            } else {
                let (first_name, last_name, company_name) = personal_info_values(
                    p.personal_info
                        .as_ref()
                        .map(|info| info.first_name.as_str()),
                    p.personal_info.as_ref().map(|info| info.last_name.as_str()),
                    p.personal_info
                        .as_ref()
                        .and_then(|info| info.company_name.as_deref()),
                );
                let first_name = display_value(&first_name);
                let last_name = display_value(&last_name);
                let company_name = display_value(&company_name);
                output::print_kv(&[
                    ("Prospect ID", &p.prospect_id),
                    ("Public ID", &p.public_id),
                    ("Email", &p.email),
                    ("First Name", &first_name),
                    ("Last Name", &last_name),
                    ("Company Name", &company_name),
                    ("Stage", &format!("{:?}", p.stage)),
                ]);
                println!("\nProspect closed.");
            }
        }
        ProspectAction::SumsubLink { prospect_id } => {
            let vars = sumsub_permalink_create::Variables {
                input: sumsub_permalink_create::SumsubPermalinkCreateInput { prospect_id },
            };
            let data = client.execute::<SumsubPermalinkCreate>(vars).await?;
            let url = data.sumsub_permalink_create.url;
            if json {
                output::print_json(&serde_json::json!({ "url": url }))?;
            } else {
                output::print_kv(&[("Sumsub URL", &url)]);
            }
        }
    }
    Ok(())
}

fn parse_customer_type(s: &str) -> Result<prospect_create::CustomerType> {
    match s.to_uppercase().as_str() {
        "INDIVIDUAL" => Ok(prospect_create::CustomerType::INDIVIDUAL),
        "GOVERNMENT_ENTITY" => Ok(prospect_create::CustomerType::GOVERNMENT_ENTITY),
        "PRIVATE_COMPANY" => Ok(prospect_create::CustomerType::PRIVATE_COMPANY),
        "BANK" => Ok(prospect_create::CustomerType::BANK),
        "FINANCIAL_INSTITUTION" => Ok(prospect_create::CustomerType::FINANCIAL_INSTITUTION),
        "FOREIGN_AGENCY_OR_SUBSIDIARY" => {
            Ok(prospect_create::CustomerType::FOREIGN_AGENCY_OR_SUBSIDIARY)
        }
        "NON_DOMICILED_COMPANY" => Ok(prospect_create::CustomerType::NON_DOMICILED_COMPANY),
        other => anyhow::bail!("Unknown customer type: {other}"),
    }
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
