use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreChartOfAccountEvent {
    ChartOfAccountsCreated,
}
