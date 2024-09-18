use lava_core::report::{dataform_client::DataformClient, upload, ReportConfig};

#[tokio::test]
#[ignore]
async fn client_test() -> anyhow::Result<()> {
    let (creds, prefix) = if let (Ok(creds), Ok(prefix)) = (
        std::env::var("TF_VAR_bq_creds"),
        std::env::var("TF_VAR_name_prefix"),
    ) {
        (creds, prefix)
    } else {
        return Ok(());
    };

    let cfg = ReportConfig::init(creds, prefix, "europe-west6".to_string())?;
    let mut client = DataformClient::connect(&cfg).await?;
    let res = client.compile().await?;
    let res = client.invoke(&res).await?;
    Ok(())
}

#[tokio::test]
async fn upload_test() -> anyhow::Result<()> {
    let (creds, prefix) = if let (Ok(creds), Ok(prefix)) = (
        std::env::var("TF_VAR_bq_creds"),
        std::env::var("TF_VAR_name_prefix"),
    ) {
        (creds, prefix)
    } else {
        return Ok(());
    };

    let cfg = ReportConfig::init(creds, prefix, "europe-west6".to_string())?;
    let all_reports = upload::bq::find_report_outputs(&cfg).await?;
    for report in all_reports {
        let rows = upload::bq::query_report(&cfg, &report).await?;
        let xml_data = upload::convert_to_xml_data(rows)?;
    }

    Ok(())
}
