use anyhow::Result;

use crate::cli::ReportAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

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
                            ("Reports", &format!("{:?}", r.reports)),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Report run not found");
                    }
                }
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
                    .map(|r| vec![r.report_run_id.clone(), format!("{:?}", r.state)])
                    .collect();
                output::print_table(&["Report Run ID", "State"], rows);
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
