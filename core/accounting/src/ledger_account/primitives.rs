use crate::primitives::{Satoshis, UsdCents};

pub enum LayeredLedgerAccountAmount {
    Usd(LayeredUsdLedgerAccountAmount),
    Btc(LayeredBtcLedgerAccountAmount),
}

pub struct LayeredUsdLedgerAccountAmount {
    pub settled: UsdLedgerAccountAmount,
    pub pending: UsdLedgerAccountAmount,
    pub encumbrance: UsdLedgerAccountAmount,
}

impl LayeredUsdLedgerAccountAmount {
    pub const ZERO: Self = Self {
        settled: UsdLedgerAccountAmount::ZERO,
        pending: UsdLedgerAccountAmount::ZERO,
        encumbrance: UsdLedgerAccountAmount::ZERO,
    };
}

pub struct UsdLedgerAccountAmount {
    pub dr_amount: UsdCents,
    pub cr_amount: UsdCents,
}

impl UsdLedgerAccountAmount {
    pub const ZERO: Self = Self {
        dr_amount: UsdCents::ZERO,
        cr_amount: UsdCents::ZERO,
    };
}

pub struct LayeredBtcLedgerAccountAmount {
    pub settled: BtcLedgerAccountAmount,
    pub pending: BtcLedgerAccountAmount,
    pub encumbrance: BtcLedgerAccountAmount,
}

impl LayeredBtcLedgerAccountAmount {
    pub const ZERO: Self = Self {
        settled: BtcLedgerAccountAmount::ZERO,
        pending: BtcLedgerAccountAmount::ZERO,
        encumbrance: BtcLedgerAccountAmount::ZERO,
    };
}

pub struct BtcLedgerAccountAmount {
    pub dr_amount: Satoshis,
    pub cr_amount: Satoshis,
}

impl BtcLedgerAccountAmount {
    pub const ZERO: Self = Self {
        dr_amount: Satoshis::ZERO,
        cr_amount: Satoshis::ZERO,
    };
}
