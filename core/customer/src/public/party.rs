use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    party::Party,
    primitives::{CustomerType, PartyId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicParty {
    pub id: PartyId,
    pub email: String,
    pub customer_type: CustomerType,
}

impl From<&Party> for PublicParty {
    fn from(entity: &Party) -> Self {
        PublicParty {
            id: entity.id,
            email: entity.email.clone(),
            customer_type: entity.customer_type,
        }
    }
}
