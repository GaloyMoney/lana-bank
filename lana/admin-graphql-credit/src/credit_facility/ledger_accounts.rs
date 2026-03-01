use async_graphql::*;

use admin_graphql_shared::primitives::*;

#[derive(SimpleObject)]
pub(super) struct CreditFacilityLedgerAccounts {
    pub facility_account_id: UUID,
    pub disbursed_receivable_not_yet_due_account_id: UUID,
    pub disbursed_receivable_due_account_id: UUID,
    pub disbursed_receivable_overdue_account_id: UUID,
    pub disbursed_defaulted_account_id: UUID,
    pub collateral_account_id: UUID,
    pub collateral_in_liquidation_account_id: UUID,
    pub liquidated_collateral_account_id: UUID,
    pub proceeds_from_liquidation_account_id: UUID,
    pub interest_receivable_not_yet_due_account_id: UUID,
    pub interest_receivable_due_account_id: UUID,
    pub interest_receivable_overdue_account_id: UUID,
    pub interest_defaulted_account_id: UUID,
    pub interest_income_account_id: UUID,
    pub fee_income_account_id: UUID,
    pub payment_holding_account_id: UUID,
    pub uncovered_outstanding_account_id: UUID,
}
