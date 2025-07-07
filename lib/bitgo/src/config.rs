use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct BitgoConfig {
    pub long_lived_token: String,
    pub enterprise_id: String,
    pub express_endpoint: String,
    pub passphrase: String,
    pub bitgo_test: bool,
}
