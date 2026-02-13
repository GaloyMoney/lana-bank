use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::{PublicCustomer, PublicProspect};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCustomerEvent {
    CustomerCreated { entity: PublicCustomer },
    CustomerEmailUpdated { entity: PublicCustomer },
    ProspectCreated { entity: PublicProspect },
    ProspectKycUpdated { entity: PublicProspect },
    ProspectClosed { entity: PublicProspect },
}
