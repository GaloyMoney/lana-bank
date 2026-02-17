use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::{Path, PathBuf};

use super::{StorageClient, error::StorageClientError};

type HmacSha256 = Hmac<Sha256>;

/// Default expiry time for signed URLs (1 hour in seconds)
const DEFAULT_EXPIRY_SECONDS: u64 = 3600;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalConfig {
    #[serde(default)]
    pub root_folder: PathBuf,
    /// Base URL for the server that will serve local files (e.g., "http://localhost:5253")
    pub server_url: String,
    /// Secret key for signing URLs - must be shared across all server instances
    pub signing_secret: String,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            root_folder: PathBuf::default(),
            server_url: "http://localhost:5253".to_string(),
            signing_secret: "dev-local-storage-signing-secret".to_string(),
        }
    }
}

pub struct LocalClient {
    root_folder: PathBuf,
    server_url: String,
    signing_secret: String,
}

impl LocalClient {
    pub fn new(config: &LocalConfig) -> Self {
        LocalClient {
            root_folder: config.root_folder.clone(),
            server_url: config.server_url.clone(),
            signing_secret: config.signing_secret.clone(),
        }
    }

    fn resolve<P: AsRef<Path>>(&self, relative: P) -> PathBuf {
        self.root_folder.join(relative)
    }

    /// Generate a signature for a path and expiry time
    fn sign(&self, path: &str, expires: u64) -> String {
        let message = format!("{}:{}", path, expires);
        let mut mac = HmacSha256::new_from_slice(self.signing_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Verify a signature for a path and expiry time (constant-time comparison)
    pub fn verify_signature(&self, path: &str, expires: u64, signature: &str) -> bool {
        let Ok(sig_bytes) = hex::decode(signature) else {
            return false;
        };
        let message = format!("{}:{}", path, expires);
        let mut mac = HmacSha256::new_from_slice(self.signing_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        mac.verify_slice(&sig_bytes).is_ok()
    }

    /// Read a file from local storage after verifying the signature
    pub async fn read_file(
        &self,
        path: &str,
        expires: u64,
        signature: &str,
    ) -> Result<Vec<u8>, StorageClientError> {
        // Check expiry
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if now > expires {
            return Err(StorageClientError::SignatureExpired);
        }

        // Verify signature
        if !self.verify_signature(path, expires, signature) {
            return Err(StorageClientError::InvalidSignature);
        }

        // Prevent path traversal
        if path.contains("..") || path.starts_with('/') {
            return Err(StorageClientError::InvalidPath);
        }

        let full_path = self.resolve(path);

        // Verify the resolved path is within root_folder
        let canonical_root = self
            .root_folder
            .canonicalize()
            .map_err(|e| StorageClientError::Other(e.into()))?;
        let canonical_path = full_path
            .canonicalize()
            .map_err(|e| StorageClientError::Other(e.into()))?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(StorageClientError::InvalidPath);
        }

        tokio::fs::read(canonical_path)
            .await
            .map_err(|e| StorageClientError::Other(e.into()))
    }
}

#[async_trait::async_trait]
impl StorageClient for LocalClient {
    async fn upload(
        &self,
        file: Vec<u8>,
        path_in_bucket: &str,
        _mime_type: &str,
    ) -> Result<(), StorageClientError> {
        let full_path = self.resolve(path_in_bucket);

        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(full_path, file).await?;
        Ok(())
    }

    async fn remove<'a>(
        &self,
        location_in_storage: super::LocationInStorage<'a>,
    ) -> Result<(), StorageClientError> {
        let full_path = self.resolve(location_in_storage.path);
        tokio::fs::remove_file(full_path).await?;
        Ok(())
    }

    async fn generate_download_link<'a>(
        &self,
        location_in_storage: super::LocationInStorage<'a>,
    ) -> Result<String, StorageClientError> {
        let path = location_in_storage.path;

        // Calculate expiry time (current time + default expiry)
        let expires = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() + DEFAULT_EXPIRY_SECONDS)
            .unwrap_or(0);

        // Generate signature
        let signature = self.sign(path, expires);

        // Encode path as base64url for safe URL transmission
        let encoded_path = URL_SAFE_NO_PAD.encode(path.as_bytes());

        Ok(format!(
            "{}/local-storage/{}?expires={}&sig={}",
            self.server_url.trim_end_matches('/'),
            encoded_path,
            expires,
            signature
        ))
    }
}
