crate::entity_id! { UserId }
crate::entity_id! { FixedTermLoanId }
crate::entity_id! { LineOfCreditContractId }
crate::entity_id! { JobId }

impl From<FixedTermLoanId> for LedgerAccountId {
    fn from(id: FixedTermLoanId) -> Self {
        LedgerAccountId::from(id.0)
    }
}
impl From<FixedTermLoanId> for JobId {
    fn from(id: FixedTermLoanId) -> Self {
        JobId::from(id.0)
    }
}

pub enum DebitOrCredit {
    Debit,
    Credit,
}

pub use cala_types::primitives::{
    AccountId as LedgerAccountId, Currency, JournalId as LedgerJournalId,
};

pub struct Money {
    pub amount: rust_decimal::Decimal,
    pub currency: Currency,
}
