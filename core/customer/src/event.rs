use serde::{Deserialize, Serialize};

use crate::primitives::{CustomerId, CustomerType, KycVerification};

#[derive(Debug, Serialize, Deserialize)]
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
