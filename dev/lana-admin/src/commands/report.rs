use anyhow::Result;

use crate::cli::ReportAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

fn format_find_extensions(files: &[find_report_run::FindReportRunReportRunReportsFiles]) -> String {
    let exts: Vec<&str> = files.iter().map(|file| file.extension.as_str()).collect();
    if exts.is_empty() {
        "none".to_string()
    } else {
        exts.join(", ")
    }
}

pub async fn execute(client: &mut GraphQLClient, action: ReportAction, json: bool) -> Result<()> {
    match action {
        ReportAction::Find { id } => {
            let vars = find_report_run::Variables { id };
            let data = client.execute::<FindReportRun>(vars).await?;
            match data.report_run {
                Some(r) => {
                    if json {
                        output::print_json(&r)?;
                    } else {
                        output::print_kv(&[
                            ("Report Run ID", &r.report_run_id),
                            ("State", &format!("{:?}", r.state)),
                            ("Run Type", &format!("{:?}", r.run_type)),
                            ("Started At", r.start_time.as_deref().unwrap_or("N/A")),
                            ("Report Count", &r.reports.len().to_string()),
                        ]);
                        println!();
                        if r.reports.is_empty() {
                            println!("Reports: none");
                        } else {
                            let rows: Vec<Vec<String>> = r
                                .reports
                                .iter()
                                .map(|report| {
                                    vec![
                                        report.name.clone(),
                                        report.report_id.clone(),
                                        format_find_extensions(&report.files),
                                    ]
                                })
                                .collect();
                            output::print_table(&["Name", "Report ID", "Files"], rows);
                        }
                    }
                }
                None => output::not_found("Report run", json),
            }
        }
        ReportAction::List { first } => {
            let vars = report_runs_list::Variables { first };
            let data = client.execute::<ReportRunsList>(vars).await?;
            let nodes = data.report_runs.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|r| {
                        vec![
                            r.report_run_id.clone(),
                            format!("{:?}", r.state),
                            format!("{:?}", r.run_type),
                            r.start_time.clone().unwrap_or_else(|| "N/A".to_string()),
                            r.reports.len().to_string(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &[
                        "Report Run ID",
                        "State",
                        "Run Type",
                        "Started At",
                        "Reports",
                    ],
                    rows,
                );
            }
        }
        ReportAction::DownloadLink {
            report_id,
            extension,
        } => {
            let vars = report_file_download_link_generate::Variables {
                input: report_file_download_link_generate::ReportFileGenerateDownloadLinkInput {
                    report_id,
                    extension,
                },
            };
            let data = client
                .execute::<ReportFileDownloadLinkGenerate>(vars)
                .await?;
            let result = data.report_file_generate_download_link;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[("URL", &result.url)]);
            }
        }
        ReportAction::Trigger {
            report_definition_id,
            as_of_date,
        } => {
            let vars = report_run_trigger::Variables {
                input: report_run_trigger::ReportRunTriggerInput {
                    report_definition_id,
                    as_of_date,
                },
            };
            let data = client.execute::<ReportRunTrigger>(vars).await?;
            let result = data.report_run_trigger.report_run;
            if json {
                output::print_json(&result)?;
            } else {
                output::print_kv(&[
                    ("Report Run ID", &result.report_run_id),
                    ("State", &format!("{:?}", result.state)),
                    ("Run Type", &format!("{:?}", result.run_type)),
                    ("Started At", result.start_time.as_deref().unwrap_or("N/A")),
                ]);
            }
        }
    }
    Ok(())
}
