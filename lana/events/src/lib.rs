#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

pub use core_access::CoreAccessEvent;
pub use core_accounting::CoreAccountingEvent;
pub use core_credit::CoreCreditCollateralEvent;
pub use core_credit::{CollateralDirection, CoreCreditEvent, ObligationStatus, ObligationType};
pub use core_credit_collection::CoreCreditCollectionEvent;
pub use core_custody::CoreCustodyEvent;
pub use core_customer::CoreCustomerEvent;
pub use core_deposit::CoreDepositEvent;
pub use core_price::CorePriceEvent;
pub use core_report::CoreReportEvent;
pub use core_time_events::CoreTimeEvent;
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
    CreditCollateral(CoreCreditCollateralEvent),
    CreditCollection(CoreCreditCollectionEvent),
    Deposit(CoreDepositEvent),
    Custody(CoreCustodyEvent),
    Report(CoreReportEvent),
    Price(CorePriceEvent),
    Time(CoreTimeEvent),
}
