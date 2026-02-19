use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::{PublicCustomer, PublicParty, PublicProspect};

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCustomerEvent {
    CustomerCreated { entity: PublicCustomer },
    CustomerKycUpdated { entity: PublicCustomer },
    CustomerActivityUpdated { entity: PublicCustomer },
    PartyCreated { entity: PublicParty },
    PartyEmailUpdated { entity: PublicParty },
    ProspectCreated { entity: PublicProspect },
    ProspectKycStarted { entity: PublicProspect },
    ProspectKycPending { entity: PublicProspect },
    ProspectKycDeclined { entity: PublicProspect },
    ProspectConverted { entity: PublicProspect },
    ProspectClosed { entity: PublicProspect },
}
