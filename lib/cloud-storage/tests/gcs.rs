use cloud_storage::{config::StorageConfig, LocationInCloud, Storage};

#[tokio::test]
async fn upload_doc() -> anyhow::Result<()> {
    if std::env::var("SA_CREDS_BASE64")
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        println!("Skipping GCS test: SA_CREDS_BASE64 not set or empty");
        return Ok(());
    }

    let config = if let Ok(name_prefix) = std::env::var("DEV_ENV_NAME_PREFIX") {
        StorageConfig::new_dev_mode(name_prefix)
    } else {
        StorageConfig {
            root_folder: "gha".to_string(),
            bucket_name: "gha-lana-documents".to_string(),
        }
    };

    let storage = Storage::new(&config);

    let content_str = "test";
    let content = content_str.as_bytes().to_vec();
    let filename = "test.txt";

    let _ = storage.upload(content, filename, "application/txt").await;
    let res = storage._list("".to_string()).await?;

    assert!(!res.is_empty());
    let count = res.len();

    // generate link
    let location = LocationInCloud {
        bucket: storage.bucket_name(),
        path_in_bucket: filename,
    };
    let link = storage.generate_download_link(location.clone()).await?;

    // download and verify the link
    let res = reqwest::get(link).await?;
    assert!(res.status().is_success());

    let return_content = res.text().await?;
    assert_eq!(return_content, content_str);

    // remove docs
    let _ = storage.remove(location).await;

    // verify list is now empty
    let res = storage._list("".to_string()).await?;
    assert_eq!(res.len(), count - 1);

    Ok(())
}
