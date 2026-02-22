use async_graphql::*;

pub use lana_app::accounting::manual_transaction::ManualEntryInput;

use admin_graphql_shared::{primitives::Date, scalars::Decimal};

use lana_app::primitives::DebitOrCredit;

use super::ledger_transaction::LedgerTransaction;

#[derive(InputObject)]
pub struct ManualTransactionExecuteInput {
    pub description: String,
    pub reference: Option<String>,
    pub effective: Option<Date>,
    pub entries: Vec<ManualTransactionEntryInput>,
}
mutation_payload! { ManualTransactionExecutePayload, transaction: LedgerTransaction }

#[derive(InputObject)]
pub struct ManualTransactionEntryInput {
    pub account_ref: String,
    pub amount: Decimal,
    pub currency: String,
    pub direction: DebitOrCredit,
    pub description: String,
}

impl TryFrom<ManualTransactionEntryInput> for ManualEntryInput {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(i: ManualTransactionEntryInput) -> Result<Self, Self::Error> {
        let mut builder = ManualEntryInput::builder();

        builder
            .currency(i.currency.parse()?)
            .account_id_or_code(i.account_ref.parse()?)
            .direction(i.direction)
            .amount(i.amount.into())
            .description(i.description);

        Ok(builder.build().expect("all fields provided"))
    }
}
