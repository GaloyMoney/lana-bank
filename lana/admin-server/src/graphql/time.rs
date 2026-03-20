use async_graphql::*;

use crate::primitives::{Date, Timestamp};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum EodProcessStatus {
    Initialized,
    AwaitingPhase1,
    Phase1Complete,
    AwaitingPhase2,
    Completed,
    Failed,
    Cancelled,
}

impl From<lana_app::time_events::EodProcessStatus> for EodProcessStatus {
    fn from(value: lana_app::time_events::EodProcessStatus) -> Self {
        match value {
            lana_app::time_events::EodProcessStatus::Initialized => Self::Initialized,
            lana_app::time_events::EodProcessStatus::AwaitingPhase1 => Self::AwaitingPhase1,
            lana_app::time_events::EodProcessStatus::Phase1Complete => Self::Phase1Complete,
            lana_app::time_events::EodProcessStatus::AwaitingPhase2 => Self::AwaitingPhase2,
            lana_app::time_events::EodProcessStatus::Completed => Self::Completed,
            lana_app::time_events::EodProcessStatus::Failed => Self::Failed,
            lana_app::time_events::EodProcessStatus::Cancelled => Self::Cancelled,
        }
    }
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
