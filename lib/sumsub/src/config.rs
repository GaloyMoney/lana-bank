use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SumsubConfig {
    #[serde(default)]
    pub sumsub_key: String,
    #[serde(default)]
    pub sumsub_secret: String,
}
