use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

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
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum EodProcessStatus {
    #[default]
    Initialized,
    AwaitingObligationsAndDeposits,
    ObligationsAndDepositsComplete,
    AwaitingCreditFacilityEod,
    Completed,
    Failed,
    Cancelled,
}

impl EodProcessStatus {
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self,
            Self::Initialized
                | Self::AwaitingObligationsAndDeposits
                | Self::ObligationsAndDepositsComplete
                | Self::AwaitingCreditFacilityEod
        )
    }
}
