use super::{config::ReportConfig, dataform_client::UploadResult};

pub async fn execute(config: &ReportConfig) -> anyhow::Result<UploadResult> {
    for table in bq::find_report_outputs(config).await? {
        // let rows = big_query::query_report(
        //     &config.gcp_project,
        //     &config.gcp_location,
        //     config.service_account_key(),
        // )
        //     let xml_bytes = convert_to_xml_data(rows)
        //     use cloud_storage
        //     Object::create(
        //         &self.bucket_name,
        //         pdf_bytes.to_vec(),
        //         &path_to_report(&self.folder_prefix, &data.debitor_company_id, &data.cn_ref),
        //         "application/pdf",
        //     )
        //     .await
        //     .map_err(err_into_status)?;
        //     let note = Object::read(
        //         &self.bucket_name,
        //         &path_to_report(&self.folder_prefix, &company_id, &cn_ref),
        //     )
        //     .await
        //     .map_err(err_into_status)?;
        //     let download_url = note.download_url(60 * 10).map_err(err_into_status)?; <- lazy
        //     evaluate the download link
    }
    Ok(UploadResult::default())
}

// fn path_to_report(docs_bucket: &str, company_id: &str, cn_ref: &str) -> String {
//     format!("{}/reports/{}/{}.xml", docs_bucket, day, table_name)
// }

pub mod bq {
    use super::*;

    use gcp_bigquery_client::{
        model::query_request::QueryRequest, table::ListOptions, yup_oauth2::ServiceAccountKey,
        Client,
    };

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

    async fn query_report(
        gcp_project: &str,
        dataset_id: &str,
        report: &str,
        creds: ServiceAccountKey,
    ) -> anyhow::Result<()> {
        let client = Client::from_service_account_key(creds, false).await?;
        let query = format!("SELECT * FROM `{}.{}.{}`", gcp_project, dataset_id, report);
        client
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
        Ok(())
    }
}
