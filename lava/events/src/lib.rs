#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};

use core_user::CoreUserEvent;
use governance::GovernanceEvent;
use outbox::OutboxEventMarker;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "module")]
pub enum LavaEvent {
    Governance(GovernanceEvent),
    User(CoreUserEvent),
    Credit(CreditEvent),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum CreditEvent {
    CreditFacilityCreated,
    CreditFacilityActivated,
    CreditFacilityCompleted,
    CreditFacilityFundsDisbursed { id: uuid::Uuid, amount: u64 },
}

macro_rules! impl_event_marker {
    ($from_type:ty, $variant:ident) => {
        impl OutboxEventMarker<$from_type> for LavaEvent {
            fn as_event(&self) -> Option<&$from_type> {
                match self {
                    Self::$variant(ref event) => Some(event),
                    _ => None,
                }
            }
        }
        impl From<$from_type> for LavaEvent {
            fn from(event: $from_type) -> Self {
                Self::$variant(event)
            }
        }
    };
}

impl_event_marker!(GovernanceEvent, Governance);
impl_event_marker!(CoreUserEvent, User);
impl_event_marker!(CreditEvent, Credit);
