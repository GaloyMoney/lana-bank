use cloud_storage::Object;

use std::collections::HashMap;

use super::{config::ReportConfig, dataform_client::UploadResult};

#[derive(Debug, Default)]
pub struct QueryRow(HashMap<String, serde_json::Value>);

pub async fn execute(config: &ReportConfig) -> anyhow::Result<UploadResult> {
    for report in bq::find_report_outputs(config).await? {
        let rows = bq::query_report(config, &report).await?;
        let xml_bytes = convert_to_xml_data(rows)?;
        Object::create(
            &config.bucket_name,
            xml_bytes.to_vec(),
            &path_to_report(&config.reports_root_folder, &report),
            "application/xml",
        )
        .await?;

        let note = Object::read(
            &config.bucket_name,
            &path_to_report(&config.reports_root_folder, &report),
        )
        .await?;

        let _download_url = note.download_url(60 * 10)?;
    }

    Ok(UploadResult::default())
}

fn path_to_report(reports_root_folder: &str, report: &str) -> String {
    let day = chrono::Utc::now().format("%Y-%m-%d").to_string();
    format!("{}/reports/{}/{}.xml", reports_root_folder, day, report)
}

pub fn convert_to_xml_data(rows: Vec<QueryRow>) -> anyhow::Result<Vec<u8>> {
    let mut xml = String::new();

    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<data>\n");

    for row in rows {
        xml.push_str("  <row>\n");
        for (key, value) in row.0 {
            let v = match value {
                serde_json::Value::String(s) => s,
                _ => String::new(),
            };
            xml.push_str(&format!("<{}>{}</{}>\n", key, v, key));
        }
        xml.push_str("</row>\n");
    }
    xml.push_str("</data>\n");

    Ok(xml.into_bytes())
}

pub mod bq {
    use super::*;

    use gcp_bigquery_client::{model::query_request::QueryRequest, table::ListOptions, Client};

    pub async fn find_report_outputs(config: &ReportConfig) -> anyhow::Result<Vec<String>> {
        let client = Client::from_service_account_key(config.service_account_key(), false).await?;
        let tables = client
            .table()
            .list(
                &config.gcp_project,
                &config.dataform_output_dataset,
                ListOptions::default(),
            )
            .await?;
        let res = tables
            .tables
            .unwrap_or_else(|| Vec::new())
            .into_iter()
            .filter_map(|t| {
                if t.table_reference.table_id.starts_with("report") {
                    return Some(t.table_reference.table_id);
                }
                None
            })
            .collect();
        Ok(res)
    }

    pub async fn query_report(
        config: &ReportConfig,
        report: &str,
    ) -> anyhow::Result<Vec<QueryRow>> {
        let client = Client::from_service_account_key(config.service_account_key(), false).await?;
        let gcp_project = &config.gcp_project;
        let query = format!(
            "SELECT * FROM `{}.{}.{}`",
            gcp_project, config.dataform_output_dataset, report
        );
        let res = client
            .job()
            .query(
                &gcp_project,
                QueryRequest {
                    query,
                    dry_run: Some(false),
                    use_legacy_sql: false,
                    ..Default::default()
                },
            )
            .await?;

        let field_names: Vec<String> = res
            .query_response()
            .schema
            .as_ref()
            .and_then(|schema| schema.fields().as_ref())
            .map(|fields| fields.iter().map(|field| field.name.clone()).collect())
            .unwrap_or_default(); // Return an em

        let rows = res
            .query_response()
            .rows
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|row| {
                let mut map = HashMap::new();
                if let Some(columns) = row.columns {
                    for (field_name, cell) in field_names.iter().zip(columns) {
                        if let Some(v) = cell.value {
                            map.insert(field_name.to_string(), v);
                        }
                    }
                }
                QueryRow(map)
            })
            .collect::<Vec<QueryRow>>();

        Ok(rows)
    }
}
