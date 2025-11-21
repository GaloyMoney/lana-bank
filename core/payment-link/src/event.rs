use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_customer::CustomerId;
use core_deposit::DepositAccountId;

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CorePaymentLinkEvent {
    FundingLinkCreated {
        id: FundingLinkId,
        customer_id: CustomerId,
        deposit_account_id: DepositAccountId,
        created_at: DateTime<Utc>,
    },
    FundingLinkActivated {
        id: FundingLinkId,
        activated_at: DateTime<Utc>,
    },
    FundingLinkDeactivated {
        id: FundingLinkId,
        deactivated_at: DateTime<Utc>,
    },
    FundingLinkBroken {
        id: FundingLinkId,
        reason: BrokenReason,
        broken_at: DateTime<Utc>,
    },
}

