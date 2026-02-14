use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    primitives::{CustomerType, KycStatus, ProspectId, ProspectStage, ProspectStatus},
    prospect::Prospect,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicProspect {
    pub id: ProspectId,
    pub email: String,
    pub customer_type: CustomerType,
    pub status: ProspectStatus,
    pub kyc_status: KycStatus,
    pub stage: ProspectStage,
}

impl From<&Prospect> for PublicProspect {
    fn from(entity: &Prospect) -> Self {
        PublicProspect {
            id: entity.id,
            email: entity.email.clone(),
            customer_type: entity.customer_type,
            status: entity.status,
            kyc_status: entity.kyc_status,
            stage: entity.stage(),
        }
    }
}
