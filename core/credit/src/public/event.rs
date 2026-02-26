use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::{
    PublicCreditFacility, PublicCreditFacilityProposal, PublicDisbursal,
    PublicInterestAccrualCycle, PublicPendingCreditFacility,
};

#[derive(Debug, Clone, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCreditEvent {
    FacilityProposalCreated {
        entity: PublicCreditFacilityProposal,
    },
    FacilityProposalConcluded {
        entity: PublicCreditFacilityProposal,
    },
    PendingCreditFacilityCollateralizationChanged {
        entity: PublicPendingCreditFacility,
    },
    PendingCreditFacilityCompleted {
        entity: PublicPendingCreditFacility,
    },
    FacilityActivated {
        entity: PublicCreditFacility,
    },
    FacilityCompleted {
        entity: PublicCreditFacility,
    },
    FacilityCollateralizationChanged {
        entity: PublicCreditFacility,
    },
    DisbursalSettled {
        entity: PublicDisbursal,
    },
    AccrualPosted {
        entity: PublicInterestAccrualCycle,
    },
    PartialLiquidationInitiated {
        entity: PublicCreditFacility,
    },
}
