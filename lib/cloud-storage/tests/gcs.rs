use cloud_storage::{config::StorageConfig, LocationInCloud, Storage};

#[tokio::test]
async fn upload_doc() -> anyhow::Result<()> {
    let gcp_creds_var = std::env::var("GOOGLE_APPLICATION_CREDENTIALS");
    let creds_file_exists = gcp_creds_var
        .map(|path| std::path::Path::new(&path).exists())
        .unwrap_or(false);

    // Skip if the GOOGLE_APPLICATION_CREDENTIALS var is not set,
    // or if it is set but the file it points to doesn't exist.
    if !creds_file_exists {
        println!("Skipping GCS test: GOOGLE_APPLICATION_CREDENTIALS not set or file missing.");
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

    Ok(())
}
