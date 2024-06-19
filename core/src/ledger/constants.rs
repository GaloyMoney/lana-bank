use uuid::{uuid, Uuid};

// Journal
pub(super) const CORE_JOURNAL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

// Integrations
pub(super) const BITFINEX_OFF_BALANCE_SHEET_INTEGRATION_ID: Uuid =
    uuid!("00000000-0000-0000-0000-200000000001");
pub(super) const BITFINEX_OFF_BALANCE_SHEET_INTEGRATION_NAME: &str =
    "Off-Balance-Sheet Bitfinex Integration";
pub(super) const BITFINEX_USDT_CASH_INTEGRATION_ID: Uuid =
    uuid!("00000000-0000-0000-0000-200000000002");
pub(super) const BITFINEX_USDT_CASH_INTEGRATION_NAME: &str = "Usdt Cash Bitfinex Integration";

// Accounts
pub(super) const _BANK_USDT_CASH_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000004");

pub(super) const BANK_SHAREHOLDER_EQUITY_CODE: &str = "BANK.SHAREHOLDER_EQUITY";
pub(super) const BANK_RESERVE_FROM_SHAREHOLDER_CODE: &str = "BANK.RESERVE_FROM_SHAREHOLDER";

// Templates
pub(super) const APPROVE_LOAN_CODE: &str = "APPROVE_LOAN";
pub(super) const INCUR_INTEREST_CODE: &str = "INCUR_INTEREST";
pub(super) const RECORD_PAYMENT_CODE: &str = "RECORD_PAYMENT";
pub(super) const COMPLETE_LOAN_CODE: &str = "COMPLETE_LOAN";
pub(super) const ADD_EQUITY_CODE: &str = "ADD_EQUITY";
