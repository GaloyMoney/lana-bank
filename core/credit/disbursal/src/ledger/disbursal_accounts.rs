#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::primitives::{CalaAccountId, ObligationReceivableAccountIds};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DisbursalLedgerAccountIds {
    receivable_not_yet_due_account_id: CalaAccountId,
    receivable_due_account_id: CalaAccountId,
    receivable_overdue_account_id: CalaAccountId,
    defaulted_account_id: CalaAccountId,
}

impl DisbursalLedgerAccountIds {
    pub fn new(
        receivable_not_yet_due_account_id: CalaAccountId,
        receivable_due_account_id: CalaAccountId,
        receivable_overdue_account_id: CalaAccountId,
        defaulted_account_id: CalaAccountId,
    ) -> Self {
        Self {
            receivable_not_yet_due_account_id,
            receivable_due_account_id,
            receivable_overdue_account_id,
            defaulted_account_id,
        }
    }

    pub fn defaulted_account_id(&self) -> CalaAccountId {
        self.defaulted_account_id
    }
}

impl From<DisbursalLedgerAccountIds> for ObligationReceivableAccountIds {
    fn from(account_ids: DisbursalLedgerAccountIds) -> Self {
        Self {
            not_yet_due: account_ids.receivable_not_yet_due_account_id,
            due: account_ids.receivable_due_account_id,
            overdue: account_ids.receivable_overdue_account_id,
        }
    }
}
