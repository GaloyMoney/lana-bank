use derive_builder::Builder;

use cala_ledger::{Currency, DebitOrCredit};
use rust_decimal::Decimal;

use crate::primitives::AccountIdOrCode;

pub use cala_ledger::TransactionId as CalaTransactionId;

#[derive(Builder)]
pub struct ManualEntryInput {
    pub(super) account_id_or_code: AccountIdOrCode,
    pub(super) amount: Decimal,
    pub(super) currency: Currency,
    #[builder(setter(into))]
    pub(super) description: String,
    pub(super) direction: DebitOrCredit,
}

impl ManualEntryInput {
    pub fn builder() -> ManualEntryInputBuilder {
        ManualEntryInputBuilder::default()
    }
}
