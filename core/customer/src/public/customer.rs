use serde::{Deserialize, Serialize};

use crate::{
    entity::Customer,
    primitives::{CustomerId, CustomerType, KycVerification},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicCustomer {
    pub id: CustomerId,
    pub email: String,
    pub customer_type: CustomerType,
    pub kyc_verification: KycVerification,
}

impl From<&Customer> for PublicCustomer {
    fn from(entity: &Customer) -> Self {
        PublicCustomer {
            id: entity.id,
            email: entity.email.clone(),
            customer_type: entity.customer_type,
            kyc_verification: entity.kyc_verification,
        }
    }
}
