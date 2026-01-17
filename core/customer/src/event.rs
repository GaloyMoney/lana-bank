use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::{CustomerId, CustomerType, KycVerification};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCustomerEvent {
    CustomerCreated {
        id: CustomerId,
        email: String,
        customer_type: CustomerType,
    },
    CustomerAccountKycVerificationUpdated {
        id: CustomerId,
        kyc_verification: KycVerification,
        customer_type: CustomerType,
    },
    CustomerEmailUpdated {
        id: CustomerId,
        email: String,
    },
}
