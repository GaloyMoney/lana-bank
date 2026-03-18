use money::Amount;

use crate::primitives::RestrictedCurrencyMap;

/// Balance for a single currency within a deposit account.
#[derive(Debug, Clone, Copy)]
pub struct DepositAccountBalance {
    pub settled: Amount,
    pub pending: Amount,
}

impl DepositAccountBalance {
    pub fn is_zero(&self) -> bool {
        self.settled.is_zero() && self.pending.is_zero()
    }
}

/// Per-currency balances for a deposit account, restricted to the account's currency scope.
pub type DepositAccountBalances = RestrictedCurrencyMap<DepositAccountBalance>;
