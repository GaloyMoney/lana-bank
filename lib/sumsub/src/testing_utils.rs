//! Sumsub testing utilities module
//! Contains constants and helper functions for testing Sumsub integration

// Test configuration constants (not duplicated in wire.rs)
#[cfg(feature = "sumsub-testing")]
pub const TEST_LEVEL_NAME: &str = "basic-kyc-level";

// Test artifact filenames
#[cfg(feature = "sumsub-testing")]
pub const PASSPORT_FILENAME: &str = "german_passport.jpg";
#[cfg(feature = "sumsub-testing")]
pub const POA_FILENAME: &str = "poa_krause_green.jpg";

// Test document URLs
#[cfg(feature = "sumsub-testing")]
pub const GERMAN_PASSPORT_URL: &str = "https://sumsub.com/files/29346237-germany-passport.jpg";
#[cfg(feature = "sumsub-testing")]
pub const POA_DOCUMENT_URL: &str = "https://sumsub.com/files/62349849-poa-krause-green.jpg";

/// Generic function to load test documents, downloading if not present locally
#[cfg(feature = "sumsub-testing")]
pub async fn load_test_document(
    filename: &str,
    download_url: &str,
    description: &str,
) -> Result<Vec<u8>, std::io::Error> {
    let artefacts_dir = "artefacts";
    let image_path = format!("{artefacts_dir}/{filename}");

    // Check if file already exists locally
    if std::path::Path::new(&image_path).exists() {
        return std::fs::read(&image_path);
    }

    // Create directory if it doesn't exist
    std::fs::create_dir_all(artefacts_dir)?;

    // Download the file
    tracing::info!("Downloading {} to {}", description, image_path);
    let response = reqwest::get(download_url)
        .await
        .map_err(std::io::Error::other)?;

    let bytes = response.bytes().await.map_err(std::io::Error::other)?;

    // Save to file
    std::fs::write(&image_path, &bytes)?;

    Ok(bytes.to_vec())
}

/// Load Sumsub configuration from environment variables
/// Returns None if required environment variables are not set
#[cfg(feature = "sumsub-testing")]
pub fn load_config_from_env() -> Option<crate::SumsubConfig> {
    let sumsub_key = std::env::var("SUMSUB_KEY").ok()?;
    let sumsub_secret = std::env::var("SUMSUB_SECRET").ok()?;

    if sumsub_key.trim().is_empty() || sumsub_secret.trim().is_empty() {
        tracing::warn!("SUMSUB_KEY and/or SUMSUB_SECRET are set environment variables but empty");
        return None;
    }    

    Some(crate::SumsubConfig {
        sumsub_key,
        sumsub_secret,
    })
}
