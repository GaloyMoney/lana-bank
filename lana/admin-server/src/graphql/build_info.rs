use async_graphql::SimpleObject;

#[derive(Clone, SimpleObject)]
pub struct BuildInfo {
    version: String,
    build_profile: String,
    build_target: String,
    enabled_features: Vec<String>,
}

impl BuildInfo {
    pub fn new(enabled_features: Vec<String>) -> Self {
        Self {
            version: env!("BUILD_VERSION").to_string(),
            build_profile: env!("BUILD_PROFILE").to_string(),
            build_target: env!("BUILD_TARGET").to_string(),
            enabled_features,
        }
    }
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
