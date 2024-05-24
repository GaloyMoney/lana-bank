use crate::primitives::{LedgerAccountId, Money};

use super::cala::graphql::*;

pub struct LedgerAccount {
    pub id: LedgerAccountId,
    pub settled_usd_balance: Option<Money>,
    pub settled_btc_balance: Option<Money>,
}

impl From<account_by_external_id::AccountByExternalIdAccountByExternalId> for LedgerAccount {
    fn from(account: account_by_external_id::AccountByExternalIdAccountByExternalId) -> Self {
        LedgerAccount {
            id: LedgerAccountId::from(account.account_id),
            settled_usd_balance: account.usd_balance.map(|b| Money {
                amount: b.settled.normal_balance.units,
                currency: b.settled.normal_balance.currency,
            }),
            settled_btc_balance: account.btc_balance.map(|b| Money {
                amount: b.settled.normal_balance.units,
                currency: b.settled.normal_balance.currency,
            }),
        }
    }
}
