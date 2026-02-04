use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::{PublicObligation, PublicPayment, PublicPaymentAllocation};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditCollectionEvent {
    PaymentReceived { entity: PublicPayment },
    PaymentAllocated { entity: PublicPaymentAllocation },
    ObligationCreated { entity: PublicObligation },
    ObligationDue { entity: PublicObligation },
    ObligationOverdue { entity: PublicObligation },
    ObligationDefaulted { entity: PublicObligation },
    ObligationCompleted { entity: PublicObligation },
}
