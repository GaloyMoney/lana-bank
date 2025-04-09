use async_graphql::*;

pub use lana_app::accounting::manual_transactions::ManualEntryInput;

use crate::graphql::primitives::*;

use cala_ledger::DebitOrCredit;

use super::ledger_transaction::LedgerTransaction;

#[derive(InputObject)]
pub struct ManualTransactionExecuteInput {
    pub description: String,
    pub reference: Option<String>,
    pub entries: Vec<ManualTransactionEntryInput>,
}
crate::mutation_payload! { ManualTransactionExecutePayload, transaction: LedgerTransaction }

#[derive(InputObject)]
pub struct ManualTransactionEntryInput {
    pub account_ref: String,
    pub amount: Decimal,
    pub currency: String,
    pub direction: DebitOrCredit,
    pub description: Option<String>,
}

impl TryFrom<ManualTransactionEntryInput> for ManualEntryInput {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(i: ManualTransactionEntryInput) -> Result<Self, Self::Error> {
        let mut builder = ManualEntryInput::builder();

        builder.currency(i.currency.parse()?);

        builder.account_id_or_code(i.account_ref.parse()?);

        builder.direction(i.direction).amount(i.amount.into());

        if let Some(description) = i.description {
            builder.description(description);
        }

        Ok(builder.build().expect("all fields provided"))
    }
}
