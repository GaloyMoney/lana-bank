use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChartOfAccountsConfig {
    #[serde(default = "default_chart_id")]
    pub primary_chart_id: Uuid,
}

impl Default for ChartOfAccountsConfig {
    fn default() -> Self {
        ChartOfAccountsConfig {
            primary_chart_id: default_chart_id(),
        }
    }
}

fn default_chart_id() -> Uuid {
    uuid!("00000000-0000-0000-0000-000000000001")
}
