use anyhow::Result;

use crate::cli::FiscalYearAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: FiscalYearAction,
    json: bool,
) -> Result<()> {
    match action {
        FiscalYearAction::List { first, after } => {
            let vars = fiscal_years_list::Variables { first, after };
            let data = client.execute::<FiscalYearsList>(vars).await?;
            let nodes = data.fiscal_years.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|fy| {
                        vec![
                            fy.id.clone(),
                            fy.fiscal_year_id.clone(),
                            fy.opened_as_of.clone(),
                            fy.is_open.to_string(),
                            fy.is_last_month_of_year_closed.to_string(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &[
                        "ID",
                        "Fiscal Year ID",
                        "Opened As Of",
                        "Is Open",
                        "Last Month Closed",
                    ],
                    rows,
                );
                let pi = data.fiscal_years.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        FiscalYearAction::CloseMonth { fiscal_year_id } => {
            let vars = fiscal_year_close_month::Variables {
                input: fiscal_year_close_month::FiscalYearCloseMonthInput { fiscal_year_id },
            };
            let data = client.execute::<FiscalYearCloseMonth>(vars).await?;
            let fy = data.fiscal_year_close_month.fiscal_year;
            if json {
                output::print_json(&fy)?;
            } else {
                output::print_kv(&[
                    ("ID", &fy.id),
                    ("Fiscal Year ID", &fy.fiscal_year_id),
                    ("Is Open", &fy.is_open.to_string()),
                    (
                        "Last Month Closed",
                        &fy.is_last_month_of_year_closed.to_string(),
                    ),
                ]);
            }
        }
        FiscalYearAction::Close { fiscal_year_id } => {
            let vars = fiscal_year_close::Variables {
                input: fiscal_year_close::FiscalYearCloseInput { fiscal_year_id },
            };
            let data = client.execute::<FiscalYearClose>(vars).await?;
            let fy = data.fiscal_year_close.fiscal_year;
            if json {
                output::print_json(&fy)?;
            } else {
                output::print_kv(&[
                    ("ID", &fy.id),
                    ("Fiscal Year ID", &fy.fiscal_year_id),
                    ("Is Open", &fy.is_open.to_string()),
                ]);
            }
        }
    }
    Ok(())
}
