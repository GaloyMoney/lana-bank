use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::{service_account::ServiceAccountConfig, storage::Storage};

use super::{config::ReportConfig, ReportError, ReportLocationInCloud};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ReportFileUpload {
    Success {
        report_name: String,
        path_in_bucket: String,
        bucket: String,
    },
    Failure {
        report_name: String,
        reason: String,
    },
}

#[derive(Debug, Default)]
pub struct QueryRow(HashMap<String, serde_json::Value>);

pub async fn execute(
    config: &ReportConfig,
    service_account: &ServiceAccountConfig,
    storage: &Storage,
) -> Result<Vec<ReportFileUpload>, ReportError> {
    let mut res = Vec::new();
    for report_name in bq::find_report_outputs(config, service_account).await? {
        let day = chrono::Utc::now().format("%Y-%m-%d").to_string();

        // TODO: remove
        let _location = ReportLocationInCloud {
            report_name: report_name.clone(),
            bucket: storage.config.bucket_name.clone(),
            path_in_bucket: path_to_report(&storage.config.root_folder, &report_name, &day),
        };
        let rows = match bq::query_report(config, service_account, &report_name, &day).await {
            Ok(rows) => rows,
            Err(e) => {
                res.push(ReportFileUpload::Failure {
                    reason: e.to_string(),
                    report_name,
                });
                continue;
            }
        };
        let xml_bytes = convert_to_xml_data(rows);

        let path_in_bucket = path_to_report(&storage.config.root_folder, &report_name, &day);

        match storage.upload(xml_bytes.to_vec(), &path_in_bucket).await {
            Ok(_) => {
                res.push(ReportFileUpload::Success {
                    path_in_bucket: path_to_report(&storage.config.root_folder, &report_name, &day),
                    report_name,
                    bucket: storage.config.bucket_name.clone(),
                });
            }
            Err(e) => res.push(ReportFileUpload::Failure {
                reason: e.to_string(),
                report_name,
            }),
        }
    }

    Ok(res)
}

fn path_to_report(root_folder: &str, report: &str, day: &str) -> String {
    format!("{}/reports/{}/{}.xml", root_folder, day, report)
}

pub fn convert_to_xml_data(rows: Vec<QueryRow>) -> Vec<u8> {
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

    xml.into_bytes()
}

pub(super) mod bq {
    use crate::service_account::ServiceAccountConfig;

    use super::*;

    use gcp_bigquery_client::{model::query_request::QueryRequest, table::ListOptions, Client};

    pub(super) async fn find_report_outputs(
        config: &ReportConfig,
        service_account: &ServiceAccountConfig,
    ) -> Result<Vec<String>, ReportError> {
        let client =
            Client::from_service_account_key(service_account.service_account_key(), false).await?;
        let tables = client
            .table()
            .list(
                &service_account.gcp_project,
                &config.dataform_output_dataset,
                ListOptions::default(),
            )
            .await?;
        let res = tables
            .tables
            .unwrap_or_default()
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

    pub(super) async fn query_report(
        config: &ReportConfig,
        service_account: &ServiceAccountConfig,
        report: &str,
        day: &str,
    ) -> Result<Vec<QueryRow>, ReportError> {
        let client =
            Client::from_service_account_key(service_account.service_account_key(), false).await?;
        let gcp_project = &service_account.gcp_project;
        let query = format!(
            "SELECT * FROM `{}.{}.{}`('{}')",
            gcp_project, config.dataform_output_dataset, report, day
        );
        let res = client
            .job()
            .query(
                gcp_project,
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
            .unwrap_or_default();

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
