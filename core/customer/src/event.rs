use serde::{Deserialize, Serialize};

use crate::primitives::{AccountStatus, CustomerId, CustomerType};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCustomerEvent {
    CustomerCreated {
        id: CustomerId,
        email: String,
        customer_type: CustomerType,
    },
    CustomerAccountStatusUpdated {
        id: CustomerId,
        status: AccountStatus,
        customer_type: CustomerType,
    },
    CustomerEmailUpdated {
        id: CustomerId,
        email: String,
    },
}
