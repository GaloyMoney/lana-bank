use serde::{Deserialize, Serialize};

es_entity::entity_id! { EodProcessId }

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum EodProcessStatus {
    #[default]
    Initialized,
    AwaitingPhase1,
    Phase1Complete,
    AwaitingPhase2,
    Completed,
    Failed,
    Cancelled,
}

impl EodProcessStatus {
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self,
            Self::Initialized | Self::AwaitingPhase1 | Self::Phase1Complete | Self::AwaitingPhase2
        )
    }
}
