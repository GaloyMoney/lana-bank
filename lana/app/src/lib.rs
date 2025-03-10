#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod accounting_init;
pub mod app;
pub mod applicant;
pub mod authorization;
pub mod balance_sheet;
pub mod cash_flow;
pub mod document;
pub mod primitives;
pub mod profit_and_loss;
pub mod report;
pub mod service_account;
pub mod statement;
pub mod storage;
pub mod terms_template;
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
        error, ChartOfAccountsIntegrationConfig, Deposit, DepositAccount, DepositAccountBalance,
        DepositAccountHistoryCursor, DepositAccountHistoryEntry, DepositsByCreatedAtCursor,
        Withdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor,
    };

    pub type Deposits =
        deposit::CoreDeposit<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod chart_of_accounts {
    pub use chart_of_accounts::{error, tree, Chart, ControlAccountDetails};

    pub type ChartOfAccounts =
        chart_of_accounts::CoreChartOfAccounts<crate::authorization::Authorization>;
}

pub mod new_chart_of_accounts {
    pub use chart_of_accounts::new::{tree, Chart};

    pub type NewChartOfAccounts =
        chart_of_accounts::new::CoreChartOfAccounts<crate::authorization::Authorization>;
}

pub mod credit_facility {
    pub use core_credit::{
        error, CollateralUpdated, CollateralizationUpdated, CoreCreditEvent,
        CreditFacilitiesCursor, CreditFacilitiesSortBy, CreditFacility,
        CreditFacilityAccountFactories, CreditFacilityBalance, CreditFacilityConfig,
        CreditFacilityHistoryEntry, CreditFacilityOmnibusAccountIds, CreditFacilityOrigination,
        CreditFacilityRepaymentInPlan, CreditFacilityStatus, Disbursal, DisbursalExecuted,
        DisbursalStatus, DisbursalsCursor, DisbursalsSortBy, FacilityCVL, FindManyCreditFacilities,
        FindManyDisbursals, IncrementalPayment, InterestAccrued, ListDirection, Payment,
        RepaymentStatus, Sort, APPROVE_CREDIT_FACILITY_PROCESS, APPROVE_DISBURSAL_PROCESS,
    };

    pub type CreditFacilities =
        core_credit::CreditFacilities<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod terms {
    pub use core_credit::{
        AnnualRatePct, CVLPct, CollateralizationState, Duration, InterestInterval,
        OneTimeFeeRatePct, TermValues,
    };
}
