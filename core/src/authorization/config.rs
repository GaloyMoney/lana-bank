use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CasbinConfig {
    #[serde(default)]
    pub db_con: String,
}
