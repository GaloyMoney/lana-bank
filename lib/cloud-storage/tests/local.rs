use cloud_storage::{Storage, config::StorageConfig};
use tempfile::TempDir;

#[tokio::test]
async fn upload_and_download_local() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let root = dir.path().to_str().unwrap().to_string();
    let config = StorageConfig::new_local(
        root,
        "http://localhost:5253".to_string(),
        "test-signing-secret".to_string(),
    );
    let storage = Storage::new(&config);

    let content_str = "localtest";
    let content = content_str.as_bytes().to_vec();
    let filename = "sub/test.txt";

    storage
        .upload(content.clone(), filename, "text/plain")
        .await?;

    // Verify signed URL is generated
    let link = storage
        .generate_download_link(cloud_storage::LocationInStorage { path: filename })
        .await?;
    assert!(link.starts_with("http://localhost:5253/local-storage/"));
    assert!(link.contains("expires="));
    assert!(link.contains("sig="));

    // Verify file was written to disk
    let file_path = dir.path().join(filename);
    let downloaded = tokio::fs::read_to_string(&file_path).await?;
    assert_eq!(downloaded, content_str);

    storage
        .remove(cloud_storage::LocationInStorage { path: filename })
        .await?;
    assert!(!file_path.exists());

    Ok(())
}
