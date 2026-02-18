use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    entity::Customer,
    primitives::{CustomerId, KycVerification, PartyId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicCustomer {
    pub id: CustomerId,
    pub party_id: PartyId,
    pub kyc_verification: KycVerification,
}

impl From<&Customer> for PublicCustomer {
    fn from(entity: &Customer) -> Self {
        PublicCustomer {
            id: entity.id,
            party_id: entity.party_id,
            kyc_verification: entity.kyc_verification,
        }
    }
}
