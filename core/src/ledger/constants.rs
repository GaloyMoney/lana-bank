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
pub(super) const BANK_USDT_CASH_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000004");
pub(super) const BANK_USDT_CASH_NAME: &str = "Bank USDT Cash";
pub(super) const BANK_USDT_CASH_CODE: &str = "BANK.USDT_CASH";

// Templates
pub(super) const PLEDGE_UNALLOCATED_COLLATERAL_CODE: &str = "PLEDGE_UNALLOCATED_COLLATERAL";
pub(super) const APPROVE_LOAN_CODE: &str = "APPROVE_LOAN";
pub(super) const INCUR_INTEREST_CODE: &str = "INCUR_INTEREST";
pub(super) const RECORD_PAYMENT_CODE: &str = "RECORD_PAYMENT";
pub(super) const COMPLETE_LOAN_CODE: &str = "COMPLETE_LOAN";
pub(super) const INITIATE_WITHDRAWAL_FROM_CHECKING_CODE: &str = "INITIATE_WITHDRAWAL_FROM_CHECKING";
pub(super) const SETTLE_WITHDRAWAL_FROM_CHECKING_CODE: &str = "SETTLE_WITHDRAWAL_FROM_CHECKING";
