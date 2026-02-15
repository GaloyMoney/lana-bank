#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod accounting_init;
pub mod app;
pub mod kyc {
    pub use core_customer::kyc::*;
    pub type CustomerKyc = core_customer::kyc::CustomerKyc<
        crate::authorization::Authorization,
        lana_events::LanaEvent,
    >;
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
    pub type Outbox = obix::Outbox<lana_events::LanaEvent>;
}
pub mod dashboard {
    pub type Dashboard = dashboard::Dashboard<crate::authorization::Authorization>;
    pub use dashboard::DashboardValues;
}

pub mod user_onboarding {
    pub use user_onboarding::config::UserOnboardingConfig;
    pub type UserOnboarding = user_onboarding::UserOnboarding<lana_events::LanaEvent>;
}

pub mod notification {
    pub use notification::config::NotificationConfig;
    pub type Notification = notification::Notification<crate::authorization::Authorization>;
    pub type EmailNotification =
        notification::email::EmailNotification<crate::authorization::Authorization>;
    pub use notification::{NotificationFromEmail, NotificationFromName};
}

pub mod access {
    pub use core_access::{Role, RoleId, UserId, config, error, permission_set, role, user};
    pub type Access = core_access::CoreAccess<crate::audit::Audit, lana_events::LanaEvent>;
}

pub mod customer {
    pub use core_customer::{
        Activity, CUSTOMER_REF_TARGET, CoreCustomerEvent, Customer, CustomerDocumentId, CustomerId,
        CustomerType, CustomersCursor, CustomersFilter, CustomersSortBy, KycLevel, KycStatus,
        KycVerification, PROSPECT_REF_TARGET, PersonalInfo, Prospect, ProspectId, ProspectStage,
        ProspectStatus, ProspectsSortBy, PublicProspect, Sort, error, prospect_cursor,
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
    pub use crate::credit::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS;
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
        ChartOfAccountsIntegrationConfig, CoreDepositEvent, DEPOSIT_ACCOUNT_ENTITY_TYPE,
        DEPOSIT_TRANSACTION_ENTITY_TYPE, Deposit, DepositAccount, DepositAccountBalance,
        DepositAccountHistoryCursor, DepositAccountHistoryEntry, DepositAccountStatus,
        DepositAccountsByCreatedAtCursor, DepositId, DepositStatus, DepositsByCreatedAtCursor,
        RequireVerifiedCustomerForAccount, WITHDRAWAL_TRANSACTION_ENTITY_TYPE, Withdrawal,
        WithdrawalId, WithdrawalStatus, WithdrawalsByCreatedAtCursor, error,
    };

    pub type Deposits =
        core_deposit::CoreDeposit<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod accounting {
    pub use core_accounting::{
        AccountCategory, AccountCode, AccountCodeSection, AccountInfo, AccountingCsvId,
        CalaAccountBalance, CalaAccountId, ChartId, CoreAccountingEvent, FiscalYearId,
        LedgerAccountId, TransactionTemplateId, chart_of_accounts, csv, error, fiscal_year,
        journal, ledger_account, ledger_transaction, manual_transaction, transaction_templates,
        {Chart, tree},
    };

    pub type Accounting = core_accounting::CoreAccounting<
        crate::authorization::Authorization,
        lana_events::LanaEvent,
    >;
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

pub mod fiscal_year {
    pub use core_accounting::fiscal_year::*;
    pub type FiscalYears = core_accounting::FiscalYears<crate::authorization::Authorization>;
}

pub mod custody {
    pub use core_custody::{
        CustodyConfig, CustodyPublisher, Wallet, WalletId, WalletNetwork, custodian, error,
    };
    pub type Custody =
        core_custody::CoreCustody<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod credit {
    pub use core_credit::{
        APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS, APPROVE_DISBURSAL_PROCESS,
        COLLATERAL_ENTITY_TYPE, CREDIT_FACILITY_ENTITY_TYPE, ChartOfAccountsIntegrationConfig,
        Collateral, CollateralSentOut, CollateralUpdated, CollateralizationUpdated,
        CoreCreditEvent, CreditConfig, CreditFacilitiesCursor, CreditFacilitiesFilters,
        CreditFacilitiesSortBy, CreditFacility, CreditFacilityApproved,
        CreditFacilityBalanceSummary, CreditFacilityHistoryEntry, CreditFacilityProposal,
        CreditFacilityProposalId, CreditFacilityProposalsByCreatedAtCursor,
        CreditFacilityRepaymentPlanEntry, CreditFacilityStatus, DISBURSAL_TRANSACTION_ENTITY_TYPE,
        Disbursal, DisbursalExecuted, DisbursalStatus, DisbursalsCursor, DisbursalsFilters,
        DisbursalsSortBy, IncrementalPayment, InterestAccrualsPosted, Liquidation,
        LiquidationsByIdCursor, LiquidationsCursor, ListDirection, Payment, PaymentAllocation,
        PendingCreditFacilitiesByCreatedAtCursor, PendingCreditFacility,
        PendingCreditFacilityCollateralizationUpdated, PendingCreditFacilityId,
        ProceedsFromLiquidationReceived, RepaymentStatus, RepaymentType, Sort, error,
    };

    pub type Credit =
        core_credit::CoreCredit<crate::authorization::Authorization, lana_events::LanaEvent>;
}

pub mod terms_template {
    pub use core_credit_terms::{
        TermsTemplate, TermsTemplateError, TermsTemplateId,
        terms_template::error as terms_template_error,
    };

    pub type TermsTemplates =
        core_credit_terms::TermsTemplates<crate::authorization::Authorization>;
}

pub mod terms {
    pub use core_credit::{
        AnnualRatePct, CVLPct, CollateralizationState, DisbursalPolicy, FacilityDuration,
        InterestInterval, ObligationDuration, OneTimeFeeRatePct,
        PendingCreditFacilityCollateralizationState, TermValues,
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

pub mod gotenberg {
    pub use gotenberg::GotenbergConfig;
}

pub mod time_events {
    pub type TimeEvents = core_time_events::TimeEvents;
    pub use core_time_events::error;
}

pub mod domain_config {
    pub use domain_config::{
        ExposedDomainConfigs, ExposedDomainConfigsReadOnly, InternalDomainConfigs,
        apply_startup_configs,
    };
}

pub mod encryption {
    pub use encryption::EncryptionConfig;
}
