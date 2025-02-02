#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod accounting_init;
pub mod app;
pub mod applicant;
pub mod authorization;
pub mod balance_sheet;
pub mod credit_facility;
// pub mod customer;
pub mod document;
pub mod price;
pub mod primitives;
pub mod profit_and_loss;
pub mod report;
pub mod service_account;
pub mod statement;
pub mod storage;
pub mod terms;
pub mod terms_template;
mod time;
pub mod trial_balance;

pub mod outbox {
    pub type Outbox = outbox::Outbox<lana_events::LanaEvent>;
}

pub mod dashboard {
    pub type Dashboard = dashboard::Dashboard<crate::authorization::Authorization>;
    pub use dashboard::DashboardValues;
}

pub mod user_onboarding {
    pub use user_onboarding::config::UserOnboardingConfig;
    pub use user_onboarding::*;
}

pub mod user {
    pub use core_user::{error, User};
    pub type Users = core_user::Users<crate::audit::Audit, lana_events::LanaEvent>;
}

pub mod customer {
    pub use core_customer::{
        error, AccountStatus, Customer, CustomersCursor, CustomersSortBy, FindManyCustomers,
        KycLevel, Sort,
    };
    pub type Customers = core_customer::Customers<crate::audit::Audit, lana_events::LanaEvent>;
}

pub mod job {
    pub use job::*;
}

pub mod governance {
    use crate::authorization::Authorization;
    use lana_events::LanaEvent;
    pub type Governance = governance::Governance<Authorization, LanaEvent>;
    pub use crate::credit_facility::{APPROVE_CREDIT_FACILITY_PROCESS, APPROVE_DISBURSAL_PROCESS};
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
        error, Deposit, DepositAccount, DepositAccountBalance, DepositsByCreatedAtCursor,
        Withdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor,
    };

    pub type Deposits =
        deposit::CoreDeposit<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod chart_of_accounts {
    pub use chart_of_accounts::tree;

    pub type ChartOfAccounts =
        chart_of_accounts::CoreChartOfAccounts<crate::authorization::Authorization>;
}
