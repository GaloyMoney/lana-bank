use async_graphql::SimpleObject;

/// Build metadata for the running admin API service.
#[derive(Clone, SimpleObject)]
pub struct BuildInfo {
    /// Application version reported by the build.
    version: String,
    /// Cargo build profile used for this binary.
    build_profile: String,
    /// Target triple the binary was built for.
    build_target: String,
    /// Cargo features compiled into the binary.
    enabled_features: Vec<String>,
}

impl BuildInfo {
    pub fn new(
        version: String,
        build_profile: String,
        build_target: String,
        enabled_features: Vec<String>,
    ) -> Self {
        Self {
            version,
            build_profile,
            build_target,
            enabled_features,
        }
    }
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self::new(
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
            Vec::new(),
        )
    }
}
