use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    primitives::{KycStatus, PartyId, ProspectId, ProspectStage},
    prospect::Prospect,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicProspect {
    pub id: ProspectId,
    pub party_id: PartyId,
    pub kyc_status: KycStatus,
    pub stage: ProspectStage,
}

impl From<&Prospect> for PublicProspect {
    fn from(entity: &Prospect) -> Self {
        PublicProspect {
            id: entity.id,
            party_id: entity.party_id,
            kyc_status: entity.kyc_status,
            stage: entity.stage,
        }
    }
}
