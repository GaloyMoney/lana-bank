use uuid::{uuid, Uuid};

// Journal
pub(super) const CORE_JOURNAL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

// Integrations
pub(super) const ON_BALANCE_SHEET_BFX_INTEGRATION_ID: Uuid =
    uuid!("00000000-0000-0000-0000-200000000000");
pub(super) const _OFF_BALANCE_SHEET_BFX_INTEGRATION_ID: Uuid =
    uuid!("10000000-0000-0000-0000-200000000000");

// Balance Sheet AccountSets
pub(super) const INTEREST_REVENUE_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-500000000001");
pub(super) const ON_BALANCE_SHEET_USER_CHECKING_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-500000000002");
pub(super) const FIXED_TERM_LOANS_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-900000000001");

// Trial Balance AccountSets
pub(super) const TRIAL_BALANCE_ACCOUNT_SET_ID: Uuid = uuid!("00000000-0000-0000-0000-110000000000");
pub(super) const USER_DEPOSITS_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-110000000001");
pub(super) const USER_CHECKING_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-110000000002");
pub(super) const FIXED_TERM_LOANS_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-110000000003");
pub(super) const INTEREST_REVENUE_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-110000000004");

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
