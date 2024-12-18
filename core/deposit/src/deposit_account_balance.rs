use core_money::UsdCents;

pub struct DepositAccountBalance {
    pub settled: UsdCents,
    pub pending: UsdCents,
}

impl DepositAccountBalance {
    pub const ZERO: Self = DepositAccountBalance {
        settled: UsdCents::ZERO,
        pending: UsdCents::ZERO,
    };
}
