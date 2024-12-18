#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_money::{Satoshis, UsdCents};
use core_user::CoreUserEvent;
use deposit::CoreDepositEvent;
use governance::GovernanceEvent;
use lana_ids::CreditFacilityId;
use outbox::OutboxEventMarker;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "module")]
pub enum LanaEvent {
    Governance(GovernanceEvent),
    User(CoreUserEvent),
    Credit(CreditEvent),
    Deposit(CoreDepositEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FacilityCollateralUpdateAction {
    Add,
    Remove,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum CreditEvent {
    FacilityCreated {
        id: CreditFacilityId,
        created_at: DateTime<Utc>,
    },
    FacilityApproved {
        id: CreditFacilityId,
    },
    FacilityActivated {
        id: CreditFacilityId,
        activated_at: DateTime<Utc>,
    },
    FacilityCompleted {
        id: CreditFacilityId,
        completed_at: DateTime<Utc>,
    },
    DisbursalExecuted {
        id: CreditFacilityId,
        amount: UsdCents,
        recorded_at: DateTime<Utc>,
    },
    FacilityRepaymentRecorded {
        id: CreditFacilityId,
        disbursal_amount: UsdCents,
        interest_amount: UsdCents,
        recorded_at: DateTime<Utc>,
    },
    FacilityCollateralUpdated {
        id: CreditFacilityId,
        new_amount: Satoshis,
        abs_diff: Satoshis,
        action: FacilityCollateralUpdateAction,
        recorded_at: DateTime<Utc>,
    },
}

macro_rules! impl_event_marker {
    ($from_type:ty, $variant:ident) => {
        impl OutboxEventMarker<$from_type> for LanaEvent {
            fn as_event(&self) -> Option<&$from_type> {
                match self {
                    Self::$variant(ref event) => Some(event),
                    _ => None,
                }
            }
        }
        impl From<$from_type> for LanaEvent {
            fn from(event: $from_type) -> Self {
                Self::$variant(event)
            }
        }
    };
}

impl_event_marker!(GovernanceEvent, Governance);
impl_event_marker!(CoreUserEvent, User);
impl_event_marker!(CreditEvent, Credit);
impl_event_marker!(CoreDepositEvent, Deposit);
