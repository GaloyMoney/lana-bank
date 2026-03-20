use money::UntypedAmount;

use crate::primitives::RestrictedCurrencyMap;

/// Balance for a single currency within a deposit account.
#[derive(Debug, Clone, Copy)]
pub struct DepositAccountBalance {
    pub settled: UntypedAmount,
    pub pending: UntypedAmount,
}

impl DepositAccountBalance {
    pub fn is_zero(&self) -> bool {
        self.settled.is_zero() && self.pending.is_zero()
    }
}

/// Per-currency balances for a deposit account, restricted to the account's currency scope.
pub type DepositAccountBalances = RestrictedCurrencyMap<DepositAccountBalance>;
