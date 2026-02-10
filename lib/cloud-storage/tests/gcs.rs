#[cfg(feature = "gcs-testing")]
fn test_storage() -> cloud_storage::Storage {
    use cloud_storage::config::StorageConfig;

    let _ = rustls::crypto::ring::default_provider().install_default();
    let bucket = std::env::var("GCS_TEST_BUCKET").unwrap_or_else(|_| "gha-lana-documents".into());
    cloud_storage::Storage::new(&StorageConfig::new_gcp(bucket))
}

#[cfg(feature = "gcs-testing")]
fn unique_path(name: &str) -> String {
    format!("test/{}/{}", uuid::Uuid::new_v4(), name)
}

#[cfg(feature = "gcs-testing")]
#[tokio::test]
#[ignore = "requires GCS credentials"]
async fn upload_and_download() -> anyhow::Result<()> {
    use cloud_storage::LocationInStorage;

    let storage = test_storage();
    let path = unique_path("test.txt");
    let content = "hello gcs integration test";

    storage
        .upload(content.as_bytes().to_vec(), &path, "text/plain")
        .await?;

    let location = LocationInStorage { path: &path };
    let link = storage.generate_download_link(location.clone()).await?;

    let res = reqwest::get(&link).await?;
    assert!(res.status().is_success());
    assert_eq!(res.text().await?, content);

    storage.remove(location).await?;

    Ok(())
}

#[cfg(feature = "gcs-testing")]
#[tokio::test]
#[ignore = "requires GCS credentials"]
async fn upload_preserves_content_type() -> anyhow::Result<()> {
    use cloud_storage::LocationInStorage;

    let storage = test_storage();
    let path = unique_path("test.pdf");
    let content = b"%PDF-1.4 fake pdf content";

    storage
        .upload(content.to_vec(), &path, "application/pdf")
        .await?;

    let location = LocationInStorage { path: &path };
    let link = storage.generate_download_link(location.clone()).await?;

    let res = reqwest::get(&link).await?;
    assert!(res.status().is_success());
    let content_type = res
        .headers()
        .get("content-type")
        .expect("missing content-type header")
        .to_str()?;
    assert!(
        content_type.contains("application/pdf"),
        "expected application/pdf, got {content_type}"
    );

    storage.remove(location).await?;

    Ok(())
}

#[cfg(feature = "gcs-testing")]
#[tokio::test]
#[ignore = "requires GCS credentials"]
async fn remove_nonexistent_file() -> anyhow::Result<()> {
    use cloud_storage::LocationInStorage;

    let storage = test_storage();
    let path = unique_path("nonexistent.txt");
    let location = LocationInStorage { path: &path };

    let result = storage.remove(location).await;
    assert!(result.is_err(), "removing a nonexistent file should error");

    Ok(())
}

#[cfg(feature = "gcs-testing")]
#[tokio::test]
#[ignore = "requires GCS credentials"]
async fn generate_download_link_for_missing_file() -> anyhow::Result<()> {
    use cloud_storage::LocationInStorage;

    let storage = test_storage();
    let path = unique_path("missing.txt");
    let location = LocationInStorage { path: &path };

    // Signed URL generation is client-side, so it succeeds even for missing objects
    let link = storage.generate_download_link(location).await?;
    let res = reqwest::get(&link).await?;
    let status = res.status().as_u16();
    assert!(
        status == 404 || status == 403,
        "fetching a signed URL for a missing object should return 404 or 403, got {status}"
    );

    Ok(())
}

#[cfg(feature = "gcs-testing")]
#[tokio::test]
#[ignore = "requires GCS credentials"]
async fn upload_nested_path() -> anyhow::Result<()> {
    use cloud_storage::LocationInStorage;

    let storage = test_storage();
    let id = uuid::Uuid::new_v4();
    let path = format!("test/{id}/a/b/c/nested.txt");
    let content = "nested content";

    storage
        .upload(content.as_bytes().to_vec(), &path, "text/plain")
        .await?;

    let location = LocationInStorage { path: &path };
    let link = storage.generate_download_link(location.clone()).await?;

    let res = reqwest::get(&link).await?;
    assert!(res.status().is_success());
    assert_eq!(res.text().await?, content);

    storage.remove(location).await?;

    Ok(())
}

#[cfg(feature = "gcs-testing")]
#[tokio::test]
#[ignore = "requires GCS credentials"]
async fn upload_binary_content() -> anyhow::Result<()> {
    use cloud_storage::LocationInStorage;

    let storage = test_storage();
    let path = unique_path("binary.bin");
    let content: Vec<u8> = (0..=255).collect();

    storage
        .upload(content.clone(), &path, "application/octet-stream")
        .await?;

    let location = LocationInStorage { path: &path };
    let link = storage.generate_download_link(location.clone()).await?;

    let res = reqwest::get(&link).await?;
    assert!(res.status().is_success());
    assert_eq!(res.bytes().await?.to_vec(), content);

    storage.remove(location).await?;

    Ok(())
}
