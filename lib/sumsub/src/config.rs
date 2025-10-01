use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SumsubConfig {
    #[serde(default)]
    pub sumsub_key: String,
    #[serde(default)]
    pub sumsub_secret: String,
}
