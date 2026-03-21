use async_graphql::*;

use crate::primitives::{Date, Timestamp, UUID};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum EodProcessStatus {
    Initialized,
    #[graphql(name = "AWAITING_OBLIGATIONS_AND_DEPOSITS")]
    AwaitingObligationsAndDeposits,
    #[graphql(name = "OBLIGATIONS_AND_DEPOSITS_COMPLETE")]
    ObligationsAndDepositsComplete,
    #[graphql(name = "AWAITING_CREDIT_FACILITY_EOD")]
    AwaitingCreditFacilityEod,
    Completed,
    Failed,
}

impl From<lana_app::eod::EodProcessStatus> for EodProcessStatus {
    fn from(value: lana_app::eod::EodProcessStatus) -> Self {
        match value {
            lana_app::eod::EodProcessStatus::Initialized => Self::Initialized,
            lana_app::eod::EodProcessStatus::AwaitingObligationsAndDeposits => {
                Self::AwaitingObligationsAndDeposits
            }
            lana_app::eod::EodProcessStatus::ObligationsAndDepositsComplete => {
                Self::ObligationsAndDepositsComplete
            }
            lana_app::eod::EodProcessStatus::AwaitingCreditFacilityEod => {
                Self::AwaitingCreditFacilityEod
            }
            lana_app::eod::EodProcessStatus::Completed => Self::Completed,
            lana_app::eod::EodProcessStatus::Failed => Self::Failed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct EndOfDayEvent {
    pub id: UUID,
    pub date: Date,
    pub status: EodProcessStatus,
}

#[derive(SimpleObject, Clone)]
pub struct Time {
    /// Current business date for the environment clock.
    current_date: Date,
    /// Current environment timestamp.
    current_time: Timestamp,
    /// Timestamp when the next end-of-day boundary will be reached.
    next_end_of_day_at: Timestamp,
    /// IANA timezone identifier for the environment (e.g. "America/New_York").
    timezone: String,
    /// Configured end-of-day time in HH:MM:SS format.
    end_of_day_time: String,
    /// Whether the environment clock can be advanced manually.
    can_advance_to_next_end_of_day: bool,
    /// Current status of the most recent end-of-day process, if any.
    eod_status: Option<EodProcessStatus>,
}

impl From<lana_app::time_events::TimeState> for Time {
    fn from(value: lana_app::time_events::TimeState) -> Self {
        Self {
            current_date: value.current_date.into(),
            current_time: value.current_time.into(),
            next_end_of_day_at: value.next_end_of_day_at.into(),
            timezone: value.timezone.to_string(),
            end_of_day_time: value.end_of_day_time.format("%H:%M:%S").to_string(),
            can_advance_to_next_end_of_day: value.can_advance_to_next_end_of_day,
            eod_status: value.eod_status.map(EodProcessStatus::from),
        }
    }
}

crate::mutation_payload! {
    TimeAdvanceToNextEndOfDayPayload,
    time: Time
}
