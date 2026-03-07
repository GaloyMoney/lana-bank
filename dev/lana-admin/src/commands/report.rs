use anyhow::Result;

use crate::cli::ReportAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

fn format_extensions(files: &[find_report_run::FindReportRunReportRunReportsFiles]) -> String {
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
                                    vec![report.report_id.clone(), format_extensions(&report.files)]
                                })
                                .collect();
                            output::print_table(&["Report ID", "Files"], rows);
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
                            r.reports.len().to_string(),
                        ]
                    })
                    .collect();
                output::print_table(&["Report Run ID", "State", "Reports"], rows);
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
        ReportAction::Trigger => {
            let vars = trigger_report_run::Variables {};
            let data = client.execute::<TriggerReportRun>(vars).await?;
            let result = data.trigger_report_run;
            if json {
                output::print_json(&result)?;
            } else {
                let run_id = result.run_id.as_deref().unwrap_or("N/A");
                output::print_kv(&[("Run ID", run_id)]);
            }
        }
    }
    Ok(())
}
