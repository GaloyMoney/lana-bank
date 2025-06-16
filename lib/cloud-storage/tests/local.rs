use cloud_storage::{
    Storage,
    config::{StorageConfig, StorageProvider},
};
use tempfile::tempdir;

#[tokio::test]
async fn upload_and_download_local() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let root = dir.path().to_str().unwrap().to_string();
    let config = StorageConfig {
        provider: StorageProvider::Local,
        root_folder: root.clone(),
        bucket_name: None,
    };
    let storage = Storage::new(&config);

    let content_str = "localtest";
    let content = content_str.as_bytes().to_vec();
    let filename = "sub/test.txt";

    storage
        .upload(content.clone(), filename, "text/plain")
        .await?;

    let link = storage
        .generate_download_link(cloud_storage::LocationInCloud {
            bucket: "",
            path_in_bucket: filename,
        })
        .await?;
    assert!(link.starts_with("file://"));
    let path = link.trim_start_matches("file://");
    let downloaded = tokio::fs::read_to_string(path).await?;
    assert_eq!(downloaded, content_str);

    storage
        .remove(cloud_storage::LocationInCloud {
            bucket: "",
            path_in_bucket: filename,
        })
        .await?;
    assert!(!std::path::Path::new(path).exists());

    Ok(())
}
