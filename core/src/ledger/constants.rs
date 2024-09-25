use uuid::{uuid, Uuid};

// Journal
pub(super) const CORE_JOURNAL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

// Reports Account Sets
pub(super) const CHART_OF_ACCOUNTS_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-100000000001");
pub(super) const TRIAL_BALANCE_ACCOUNT_SET_ID: Uuid = uuid!("00000000-0000-0000-0000-100000000002");
pub(super) const BALANCE_SHEET_ACCOUNT_SET_ID: Uuid = uuid!("00000000-0000-0000-0000-100000000003");
pub(super) const NET_INCOME_ACCOUNT_SET_ID: Uuid = uuid!("00000000-0000-0000-0000-100000000004");
pub(super) const CASH_FLOW_ACCOUNT_SET_ID: Uuid = uuid!("00000000-0000-0000-0000-100000000005");

pub(super) const OBS_CHART_OF_ACCOUNTS_ACCOUNT_SET_ID: Uuid =
    uuid!("10000000-0000-0000-0000-100000000001");
pub(super) const OBS_TRIAL_BALANCE_ACCOUNT_SET_ID: Uuid =
    uuid!("10000000-0000-0000-0000-100000000002");

// Account Sets
pub(super) const LOANS_PRINCIPAL_RECEIVABLE_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-110000000001");
pub(super) const LOANS_INTEREST_RECEIVABLE_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-110000000002");
pub(super) const OBS_CREDIT_FACILITY_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-110000000003");
pub(super) const CUSTOMER_CHECKING_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-120000000001");
pub(super) const INTEREST_REVENUE_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-140000000001");
pub(super) const LOANS_COLLATERAL_CONTROL_ACCOUNT_SET_ID: Uuid =
    uuid!("00000000-0000-0000-0000-210000000002");

// Accounts for templates
pub(super) const OBS_ASSETS_ACCOUNT_CODE: &str = "BANK.COLLATERAL.OMNIBUS";
pub(super) const BANK_DEPOSITS_OMNIBUS_CODE: &str = "BANK.DEPOSITS.OMNIBUS";
pub(super) const OBS_CREDIT_FACILITY_ACCOUNT_CODE: &str = "BANK_CREDIT_FACILITY.OMNIBUS";
pub(super) const BANK_SHAREHOLDER_EQUITY_CODE: &str = "BANK.SHAREHOLDER_EQUITY";
pub(super) const BANK_RESERVE_FROM_SHAREHOLDER_CODE: &str = "BANK.RESERVE_FROM_SHAREHOLDER";

// Templates
pub(super) const DEPOSIT_CHECKING: &str = "DEPOSIT_CHECKING";
pub(super) const INITIATE_WITHDRAW: &str = "INITIATE_WITHDRAW";
pub(super) const CONFIRM_WITHDRAW: &str = "CONFIRM_WITHDRAW";
pub(super) const CANCEL_WITHDRAW: &str = "CANCEL_WITHDRAW";
pub(super) const APPROVE_LOAN_CODE: &str = "APPROVE_LOAN";
pub(super) const APPROVE_CREDIT_FACILITY_CODE: &str = "APPROVE_CREDIT_FACILITY";
pub(super) const INCUR_INTEREST_CODE: &str = "INCUR_INTEREST";
pub(super) const RECORD_PAYMENT_CODE: &str = "RECORD_PAYMENT";
pub(super) const ADD_EQUITY_CODE: &str = "ADD_EQUITY";
pub(super) const ADD_COLLATERAL_CODE: &str = "ADD_COLLATERAL";
pub(super) const REMOVE_COLLATERAL_CODE: &str = "REMOVE_COLLATERAL";
