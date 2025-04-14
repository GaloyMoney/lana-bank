#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod accounting_init;
pub mod app;
pub mod applicant;
pub mod authorization;
pub mod balance_sheet;
pub mod cash_flow;
pub mod document;
pub mod general_ledger;
pub mod primitives;
pub mod report;
pub mod service_account;
pub mod statement;
pub mod terms_template;
pub mod trial_balance;

pub mod storage {
    pub use cloud_storage::*;
}

pub mod outbox {
    pub type Outbox = outbox::Outbox<lana_events::LanaEvent>;
}
pub mod dashboard {
    pub type Dashboard = dashboard::Dashboard<crate::authorization::Authorization>;
    pub use dashboard::DashboardValues;
}

pub mod user_onboarding {
    pub use user_onboarding::config::UserOnboardingConfig;
    pub type UserOnboarding =
        user_onboarding::UserOnboarding<crate::audit::Audit, lana_events::LanaEvent>;
}

pub mod user {
    pub use core_user::{error, User};
    pub type Users = core_user::Users<crate::audit::Audit, lana_events::LanaEvent>;
}

pub mod customer {
    pub use core_customer::{
        error, AccountStatus, Customer, CustomerId, CustomerType, CustomersCursor, CustomersSortBy,
        FindManyCustomers, KycLevel, Sort,
    };
    pub type Customers =
        core_customer::Customers<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod customer_onboarding {
    pub use customer_onboarding::config::CustomerOnboardingConfig;
    pub type CustomerOnboarding = customer_onboarding::CustomerOnboarding<
        crate::authorization::Authorization,
        lana_events::LanaEvent,
    >;
}

pub mod price {
    pub use core_price::*;
}

pub mod job {
    pub use job::*;
}

pub mod governance {
    use crate::authorization::Authorization;
    use lana_events::LanaEvent;
    pub type Governance = governance::Governance<Authorization, LanaEvent>;
    pub use crate::credit_facility::APPROVE_CREDIT_FACILITY_PROCESS;
    pub use crate::credit_facility::APPROVE_DISBURSAL_PROCESS;
    pub use deposit::APPROVE_WITHDRAWAL_PROCESS;
}

pub mod audit {
    use crate::{
        authorization::{LanaAction, LanaObject},
        primitives::Subject,
    };

    pub use audit::{error, AuditCursor, AuditEntryId, AuditInfo, AuditSvc};
    pub type Audit = audit::Audit<Subject, LanaObject, LanaAction>;
    pub type AuditEntry = audit::AuditEntry<Subject, LanaObject, LanaAction>;
}

pub mod deposit {
    pub use deposit::{
        error, ChartOfAccountsIntegrationConfig, CoreDepositEvent, Deposit, DepositAccount,
        DepositAccountBalance, DepositAccountHistoryCursor, DepositAccountHistoryEntry, DepositId,
        DepositsByCreatedAtCursor, Withdrawal, WithdrawalId, WithdrawalStatus,
        WithdrawalsByCreatedAtCursor,
    };

    pub type Deposits =
        deposit::CoreDeposit<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod accounting {
    pub use core_accounting::{
        chart_of_accounts, error, journal, ledger_account, ledger_transaction, manual_transaction,
        AccountCode, CalaAccountId, LedgerAccountId, {tree, Chart},
    };

    pub type Accounting = core_accounting::CoreAccounting<crate::authorization::Authorization>;
    pub type ChartOfAccounts =
        core_accounting::ChartOfAccounts<crate::authorization::Authorization>;
}

pub mod profit_and_loss {
    pub use core_accounting::profit_and_loss::*;
    pub type ProfitAndLossStatements =
        core_accounting::ProfitAndLossStatements<crate::authorization::Authorization>;
}

pub mod credit_facility {
    pub use core_credit::{
        error, ChartOfAccountsIntegrationConfig, CollateralUpdated, CollateralizationUpdated,
        CoreCreditEvent, CreditFacilitiesCursor, CreditFacilitiesSortBy, CreditFacility,
        CreditFacilityBalance, CreditFacilityConfig, CreditFacilityHistoryEntry,
        CreditFacilityOrigination, CreditFacilityRepaymentInPlan, CreditFacilityStatus, Disbursal,
        DisbursalExecuted, DisbursalStatus, DisbursalsCursor, DisbursalsSortBy, FacilityCVL,
        FindManyCreditFacilities, FindManyDisbursals, IncrementalPayment, InterestAccrualsPosted,
        ListDirection, Payment, RepaymentStatus, Sort, APPROVE_CREDIT_FACILITY_PROCESS,
        APPROVE_DISBURSAL_PROCESS,
    };

    pub type CreditFacilities =
        core_credit::CreditFacilities<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod terms {
    pub use core_credit::{
        AnnualRatePct, CVLPct, CollateralizationState, Duration, InterestDuration,
        InterestInterval, OneTimeFeeRatePct, TermValues,
    };
}
