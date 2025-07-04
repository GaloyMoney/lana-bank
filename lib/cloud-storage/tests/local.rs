use cloud_storage::{Storage, config::StorageConfig};
use tempfile::TempDir;

#[tokio::test]
async fn upload_and_download_local() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let root = dir.path().to_str().unwrap().to_string();
    let config = StorageConfig::new_local(root);
    let storage = Storage::new(&config);

    let content_str = "localtest";
    let content = content_str.as_bytes().to_vec();
    let filename = "sub/test.txt";

    storage
        .upload(content.clone(), filename, "text/plain")
        .await?;

    let link = storage
        .generate_download_link(cloud_storage::LocationInStorage { path: filename })
        .await?;
    assert!(link.starts_with("file://"));

    let path = link.trim_start_matches("file://");
    let downloaded = tokio::fs::read_to_string(path).await?;
    assert_eq!(downloaded, content_str);

    storage
        .remove(cloud_storage::LocationInStorage { path: filename })
        .await?;
    assert!(!std::path::Path::new(path).exists());

    Ok(())
}
