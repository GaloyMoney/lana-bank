use async_graphql::SimpleObject;

#[derive(Clone, SimpleObject)]
pub struct BuildInfo {
    version: String,
    build_profile: String,
    build_target: String,
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
