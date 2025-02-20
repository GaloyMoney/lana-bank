use chart_of_accounts::TransactionAccountFactory;

use crate::primitives::LedgerAccountId;

#[derive(Clone)]
pub struct DepositAccountFactories {
    pub deposits: TransactionAccountFactory,
}

#[derive(Clone)]
pub struct DepositOmnibusAccountIds {
    pub deposits: LedgerAccountId,
}
