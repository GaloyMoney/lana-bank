use lava_core::report::{dataform_client::DataformClient, ReportConfig};

#[tokio::test]
async fn client_test() -> anyhow::Result<()> {
    let (creds, prefix) = if let (Ok(creds), Ok(prefix)) = (
        std::env::var("TF_VAR_bq_creds"),
        std::env::var("TF_VAR_name_prefix"),
    ) {
        (creds, prefix)
    } else {
        return Ok(());
    };

    let cfg = ReportConfig {
        sa_creds_base64: creds,
        gcp_project: "cala-enterprise".to_string(),
        gcp_location: "europe-west6".to_string(),
        dataform_repo: format!("{}-repo", prefix),
        dataform_release_config: format!("{}-release", prefix),
    };
    let mut client = DataformClient::connect(&cfg).await?;
    let res = client.compile().await?;
    let res = client.invoke(&res).await?;
    Ok(())
}
