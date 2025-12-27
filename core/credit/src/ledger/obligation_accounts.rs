#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

use crate::primitives::ObligationStatus;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ObligationReceivableAccountIds {
    pub not_yet_due: CalaAccountId,
    pub due: CalaAccountId,
    pub overdue: CalaAccountId,
}

impl ObligationReceivableAccountIds {
    #[allow(clippy::new_without_default)]
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            not_yet_due: CalaAccountId::new(),
            due: CalaAccountId::new(),
            overdue: CalaAccountId::new(),
        }
    }

    pub fn id_for_status(&self, status: ObligationStatus) -> Option<CalaAccountId> {
        match status {
            ObligationStatus::NotYetDue => Some(self.not_yet_due),
            ObligationStatus::Due => Some(self.due),
            ObligationStatus::Overdue | ObligationStatus::Defaulted => Some(self.overdue),
            ObligationStatus::Paid => None,
        }
    }
}
