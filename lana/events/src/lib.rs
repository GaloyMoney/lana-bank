#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

pub use core_access::CoreAccessEvent;
pub use core_accounting::CoreAccountingEvent;
pub use core_credit::{CollateralAction, CoreCreditEvent, ObligationStatus, ObligationType};
pub use core_custody::CoreCustodyEvent;
pub use core_customer::CoreCustomerEvent;
pub use core_deposit::CoreDepositEvent;
pub use core_price::CorePriceEvent;
pub use core_report::CoreReportEvent;
pub use core_time_events::CoreTimeEvent;
pub use domain_config::CoreDomainConfigEvent;
pub use governance::GovernanceEvent;
pub use obix::out::OutboxEventMarker;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr, obix::OutboxEvent)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "module")]
pub enum LanaEvent {
    Governance(GovernanceEvent),
    CoreAccess(CoreAccessEvent),
    Accounting(CoreAccountingEvent),
    Customer(CoreCustomerEvent),
    Credit(CoreCreditEvent),
    Deposit(CoreDepositEvent),
    Custody(CoreCustodyEvent),
    Report(CoreReportEvent),
    Price(CorePriceEvent),
    Time(CoreTimeEvent),
    DomainConfig(CoreDomainConfigEvent),
}

macro_rules! impl_event_marker {
    ($from_type:ty, $variant:ident) => {
        impl OutboxEventMarker<$from_type> for LanaEvent {
            fn as_event(&self) -> Option<&$from_type> {
                match self {
                    &Self::$variant(ref event) => Some(event),
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
impl_event_marker!(CoreAccessEvent, CoreAccess);
impl_event_marker!(CoreAccountingEvent, Accounting);
impl_event_marker!(CoreCreditEvent, Credit);
impl_event_marker!(CoreDepositEvent, Deposit);
impl_event_marker!(CoreCustomerEvent, Customer);
impl_event_marker!(CoreCustodyEvent, Custody);
impl_event_marker!(CoreReportEvent, Report);
impl_event_marker!(CorePriceEvent, Price);
impl_event_marker!(CoreTimeEvent, Time);
impl_event_marker!(CoreDomainConfigEvent, DomainConfig);
