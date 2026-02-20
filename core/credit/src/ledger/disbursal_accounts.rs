#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger_core_types::primitives::AccountId as CalaAccountId;

use super::CreditFacilityLedgerAccountIds;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DisbursalLedgerAccountIds {
    receivable_not_yet_due_account_id: CalaAccountId,
    receivable_due_account_id: CalaAccountId,
    receivable_overdue_account_id: CalaAccountId,
    defaulted_account_id: CalaAccountId,
}

impl From<CreditFacilityLedgerAccountIds> for DisbursalLedgerAccountIds {
    fn from(credit_facility_account_ids: CreditFacilityLedgerAccountIds) -> Self {
        Self {
            receivable_not_yet_due_account_id: credit_facility_account_ids
                .disbursed_receivable_not_yet_due_account_id,
            receivable_due_account_id: credit_facility_account_ids
                .disbursed_receivable_due_account_id,
            receivable_overdue_account_id: credit_facility_account_ids
                .disbursed_receivable_overdue_account_id,
            defaulted_account_id: credit_facility_account_ids.disbursed_defaulted_account_id,
        }
    }
}

impl From<DisbursalLedgerAccountIds> for core_credit_collection::ObligationReceivableAccountIds {
    fn from(account_ids: DisbursalLedgerAccountIds) -> Self {
        Self {
            not_yet_due: account_ids.receivable_not_yet_due_account_id,
            due: account_ids.receivable_due_account_id,
            overdue: account_ids.receivable_overdue_account_id,
        }
    }
}

impl DisbursalLedgerAccountIds {
    pub fn defaulted_account_id(&self) -> CalaAccountId {
        self.defaulted_account_id
    }
}
