use money::CurrencyBag;

pub struct DepositAccountBalance {
    pub settled: CurrencyBag,
    pub pending: CurrencyBag,
}

impl DepositAccountBalance {
    pub fn is_zero(&self) -> bool {
        self.settled.is_empty() && self.pending.is_empty()
    }
}
