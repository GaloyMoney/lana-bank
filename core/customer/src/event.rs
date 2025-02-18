use serde::{Deserialize, Serialize};

use crate::primitives::{AccountStatus, CustomerId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCustomerEvent {
    CustomerCreated {
        id: CustomerId,
        email: String,
    },
    CustomerAccountStatusUpdated {
        id: CustomerId,
        status: AccountStatus,
    },
}
