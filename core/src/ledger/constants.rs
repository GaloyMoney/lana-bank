use uuid::{uuid, Uuid};

// Entity
pub(super) const BANK_ID: Uuid = uuid!("10000000-0000-0000-0000-000000000001");

// Journal
pub(super) const CORE_JOURNAL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

// Accounts
pub(super) const BANK_OFF_BALANCE_SHEET_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000002");
pub(super) const BANK_OFF_BALANCE_SHEET_NAME: &str = "Bank Off-Balance-Sheet Omnibus";
pub(super) const BANK_OFF_BALANCE_SHEET_CODE: &str = "BANK.OFF_BALANCE_SHEET";

pub(super) const BANK_USDT_CASH_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000004");
pub(super) const BANK_USDT_CASH_NAME: &str = "Bank USDT Cash";
pub(super) const BANK_USDT_CASH_CODE: &str = "BANK.USDT_CASH";

pub(super) const BANK_SHAREHOLDER_EQUITY_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000005");
pub(super) const BANK_SHAREHOLDER_EQUITY_NAME: &str = "Bank Shareholder Equity";
pub(super) const BANK_SHAREHOLDER_EQUITY_CODE: &str = "BANK.SHAREHOLDER_EQUITY";

// Templates
pub(super) const PLEDGE_UNALLOCATED_COLLATERAL_CODE: &str = "PLEDGE_UNALLOCATED_COLLATERAL";
pub(super) const DEPOSIT_CHECKING_CODE: &str = "DEPOSIT_CHECKING";
pub(super) const APPROVE_LOAN_CODE: &str = "APPROVE_LOAN";
pub(super) const INCUR_INTEREST_CODE: &str = "INCUR_INTEREST";
pub(super) const RECORD_PAYMENT_CODE: &str = "RECORD_PAYMENT";
pub(super) const COMPLETE_LOAN_CODE: &str = "COMPLETE_LOAN";
pub(super) const INITIATE_WITHDRAWAL_FROM_CHECKING_CODE: &str = "INITIATE_WITHDRAWAL_FROM_CHECKING";
pub(super) const SETTLE_WITHDRAWAL_FROM_CHECKING_CODE: &str = "SETTLE_WITHDRAWAL_FROM_CHECKING";
