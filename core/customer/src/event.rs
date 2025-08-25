use serde::{Deserialize, Serialize};

use crate::primitives::{CustomerId, CustomerKycStatus, CustomerType};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCustomerEvent {
    CustomerCreated {
        id: CustomerId,
        email: String,
        customer_type: CustomerType,
    },
    CustomerAccountKycStatusUpdated {
        id: CustomerId,
        kyc_status: CustomerKycStatus,
        customer_type: CustomerType,
    },
    CustomerEmailUpdated {
        id: CustomerId,
        email: String,
    },
}
