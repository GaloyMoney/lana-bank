use money::UsdCents;

pub struct DepositAccountBalance {
    pub settled: UsdCents,
    pub pending: UsdCents,
}

impl DepositAccountBalance {
    pub const ZERO: Self = DepositAccountBalance {
        settled: UsdCents::ZERO,
        pending: UsdCents::ZERO,
    };

    pub fn is_zero(&self) -> bool {
        self.settled.is_zero() && self.pending.is_zero()
    }
}
