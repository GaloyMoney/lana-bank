use serde::{Deserialize, Serialize};

use super::PublicEodProcess;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum CoreEodEvent {
    EodProcessStarted { entity: PublicEodProcess },
    EodProcessCompleted { entity: PublicEodProcess },
    EodProcessFailed { entity: PublicEodProcess },
}
