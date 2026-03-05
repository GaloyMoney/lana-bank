use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SumsubConfig {
    #[serde(default)]
    pub sumsub_key: String,
    #[serde(default)]
    pub sumsub_secret: String,
}

impl std::fmt::Debug for SumsubConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SumsubConfig")
            .field("sumsub_key", &self.sumsub_key)
            .field("sumsub_secret", &"<redacted>")
            .finish()
    }
}
