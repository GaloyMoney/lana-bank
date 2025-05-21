#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};

pub use core_access::CoreAccessEvent;
pub use core_credit::{CollateralAction, CoreCreditEvent, ObligationStatus, ObligationType};
pub use core_customer::CoreCustomerEvent;
pub use deposit::CoreDepositEvent;
pub use governance::GovernanceEvent;
pub use outbox::OutboxEventMarker;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "module")]
pub enum LanaEvent {
    Governance(GovernanceEvent),
    Access(CoreAccessEvent),
    Customer(CoreCustomerEvent),
    Credit(CoreCreditEvent),
    Deposit(CoreDepositEvent),
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
impl_event_marker!(CoreAccessEvent, Access);
impl_event_marker!(CoreCreditEvent, Credit);
impl_event_marker!(CoreDepositEvent, Deposit);
impl_event_marker!(CoreCustomerEvent, Customer);
