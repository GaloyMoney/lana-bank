#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod accounting_init;
pub mod app;

pub mod applicant {
    pub use core_applicant::*;
    pub type Applicants =
        core_applicant::Applicants<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod authorization;
pub mod primitives;

pub mod storage {
    pub use cloud_storage::*;
}

pub mod document {
    pub use document_storage::{
        Document, DocumentId, DocumentRepo, DocumentStatus, DocumentType,
        DocumentsByCreatedAtCursor, GeneratedDocumentDownloadLink, NewDocument, ReferenceId, error,
    };
    pub type DocumentStorage = document_storage::DocumentStorage;
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

pub mod notification {
    pub use notification::config::NotificationConfig;
    pub type Notification = notification::Notification;
    pub type EmailNotification = notification::email::EmailNotification;
}

pub mod rbac {
    pub use rbac_types::PermissionSetName;
}
pub mod access {
    pub use core_access::{
        AuthenticationId, Role, RoleId, UserId, config, error, permission_set, role, user,
    };
    pub type Access = core_access::CoreAccess<crate::audit::Audit, lana_events::LanaEvent>;
}

pub mod customer {
    pub use core_customer::{
        AccountStatus, CUSTOMER_REF_TARGET, Customer, CustomerDocumentId, CustomerId, CustomerType,
        CustomersCursor, CustomersSortBy, FindManyCustomers, KycLevel, Sort, error,
    };
    pub type Customers =
        core_customer::Customers<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod customer_sync {
    pub use customer_sync::config::CustomerSyncConfig;
    pub type CustomerSync =
        customer_sync::CustomerSync<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod deposit_sync {
    pub use deposit_sync::config::DepositSyncConfig;
    pub type DepositSync =
        deposit_sync::DepositSync<crate::authorization::Authorization, lana_events::LanaEvent>;
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
    pub use crate::credit::APPROVE_CREDIT_FACILITY_PROCESS;
    pub use crate::credit::APPROVE_DISBURSAL_PROCESS;
    pub use core_deposit::APPROVE_WITHDRAWAL_PROCESS;
    pub use governance::{
        ApprovalProcess, ApprovalProcessStatus, ApprovalProcessType, ApprovalRules, Committee,
        CommitteeId, Policy, approval_process_cursor, committee_cursor, error, policy_cursor,
    };
}

pub mod audit {
    use crate::{
        authorization::{LanaAction, LanaObject},
        primitives::Subject,
    };

    pub use audit::{AuditCursor, AuditEntryId, AuditInfo, AuditSvc, error};
    pub type Audit = audit::Audit<Subject, LanaObject, LanaAction>;
    pub type AuditEntry = audit::AuditEntry<Subject, LanaObject, LanaAction>;
}

pub mod deposit {
    pub use core_deposit::{
        ChartOfAccountsIntegrationConfig, CoreDepositEvent, Deposit, DepositAccount,
        DepositAccountBalance, DepositAccountHistoryCursor, DepositAccountHistoryEntry, DepositId,
        DepositStatus, DepositsByCreatedAtCursor, Withdrawal, WithdrawalId, WithdrawalStatus,
        WithdrawalsByCreatedAtCursor, error,
    };

    pub type Deposits =
        core_deposit::CoreDeposit<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod accounting {
    pub use core_accounting::{
        AccountCode, AccountCodeSection, AccountingCsvId, CalaAccountBalance, CalaAccountId, ChartId, LedgerAccountId,
        TransactionTemplateId, chart_of_accounts, csv, error, journal, ledger_account,
        ledger_transaction, manual_transaction, transaction_templates, {Chart, tree},
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

pub mod balance_sheet {
    pub use core_accounting::balance_sheet::*;
    pub type BalanceSheets = core_accounting::BalanceSheets<crate::authorization::Authorization>;
}

pub mod trial_balance {
    pub use core_accounting::trial_balance::*;
    pub type TrialBalances = core_accounting::TrialBalances<crate::authorization::Authorization>;
}

pub mod custody {
    pub use core_custody::{CustodyConfig, CustodyPublisher, Wallet, WalletId, custodian, error};
    pub type Custody =
        core_custody::CoreCustody<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod credit {
    pub use core_credit::{
        APPROVE_CREDIT_FACILITY_PROCESS, APPROVE_DISBURSAL_PROCESS,
        ChartOfAccountsIntegrationConfig, Collateral, CollateralUpdated, CollateralizationUpdated,
        CoreCreditEvent, CreditConfig, CreditFacilitiesCursor, CreditFacilitiesSortBy,
        CreditFacility, CreditFacilityApproved, CreditFacilityBalanceSummary,
        CreditFacilityHistoryEntry, CreditFacilityRepaymentPlanEntry, CreditFacilityStatus,
        Disbursal, DisbursalExecuted, DisbursalStatus, DisbursalsCursor, DisbursalsSortBy,
        FacilityCVL, FindManyCreditFacilities, FindManyDisbursals, IncrementalPayment,
        InterestAccrualsPosted, ListDirection, ObligationInstallment, ObligationMovedToLiquidation,
        Payment, RepaymentStatus, RepaymentType, Sort, TermsTemplate, error, terms_template_error,
    };

    pub type Credit =
        core_credit::CoreCredit<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod terms {
    pub use core_credit::{
        AnnualRatePct, CVLPct, CollateralizationState, FacilityDuration, InterestInterval,
        ObligationDuration, OneTimeFeeRatePct, TermValues,
    };
}

pub mod public_id {
    pub use public_id::{PublicId, PublicIdEntity, PublicIdTargetType, PublicIds, error};
}

pub mod report {
    pub use core_report::{
        CoreReportAction, CoreReportEvent, Report, ReportConfig, ReportError, ReportFile, ReportId,
        ReportObject, ReportRun, ReportRunId, ReportRunState, ReportRunType,
        ReportRunsByCreatedAtCursor, ReportsByCreatedAtCursor, error,
    };
    pub type Reports =
        core_report::CoreReports<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod contract_creation {
    pub use contract_creation::*;
    pub type ContractCreation = contract_creation::ContractCreation<
        crate::authorization::Authorization,
        lana_events::LanaEvent,
    >;
}
