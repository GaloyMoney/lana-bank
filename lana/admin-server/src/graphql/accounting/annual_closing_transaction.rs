use async_graphql::*;

use super::ledger_transaction::LedgerTransaction;
use crate::primitives::*;

#[derive(InputObject)]
pub struct AnnualClosingTransactionExecuteInput {
    pub chart_id: UUID,
}

crate::mutation_payload! {
    AnnualClosingTransactionExecutePayload,
    annual_closing_transaction: LedgerTransaction
}
