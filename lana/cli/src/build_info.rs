use serde::{Deserialize, Serialize};

/// Information about how the binary was compiled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// List of enabled Cargo features during compilation
    pub enabled_features: Vec<String>,
    /// Build profile (debug, release, etc.)
    pub build_profile: String,
    /// Target architecture triple
    pub build_target: String,
    /// Host architecture triple  
    pub build_host: String,
    /// Build timestamp
    pub build_timestamp: String,
    /// Version information
    pub version: String,
}

impl BuildInfo {
    /// Get the build information for this binary
    pub fn get() -> Self {
        let features_str = env!("ENABLED_FEATURES");
        let enabled_features = if features_str.is_empty() {
            vec![]
        } else {
            features_str.split(',').map(|s| s.to_string()).collect()
        };

        Self {
            enabled_features,
            build_profile: env!("BUILD_PROFILE").to_string(),
            build_target: env!("BUILD_TARGET").to_string(),
            build_host: env!("BUILD_HOST").to_string(),
            build_timestamp: env!("BUILD_TIMESTAMP").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Format build info for display in CLI
    pub fn display(&self) -> String {
        format!(
            "lana-cli {}\n\
             Build Profile: {}\n\
             Build Target: {}\n\
             Build Host: {}\n\
             Build Timestamp: {}\n\
             Enabled Features: [{}]",
            self.version,
            self.build_profile,
            self.build_target,
            self.build_host,
            self.build_timestamp,
            self.enabled_features.join(", ")
        )
    }
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self::get()
    }
}
