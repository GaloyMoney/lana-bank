use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct DataExportConfig {
    #[serde(default)]
    pub dev_disable_entity_events: bool,
}
