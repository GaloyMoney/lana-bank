#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod action;
mod audit_action;
mod audit_object;
mod object;

pub use action::*;
pub use audit_action::*;
pub use audit_object::*;
pub use object::*;

// Re-export Subject from core-ids (the canonical definition)
pub use core_ids::Subject;

// Re-export SystemActor from audit crate
pub use audit::SystemActor;

pub const ROLE_NAME_ACCOUNTANT: &str = "accountant";
pub const ROLE_NAME_ADMIN: &str = "admin";
pub const ROLE_NAME_BANK_MANAGER: &str = "bank-manager";

#[derive(Clone, PartialEq, Eq, Copy, async_graphql::Enum)]
pub enum PermissionSetName {
    AccessViewer,
    AccessWriter,
    AccountingViewer,
    AccountingWriter,
    CollectionViewer,
    CollectionWriter,
    CollectionPaymentDate,
    ContractCreation,
    CreditViewer,
    CreditWriter,
    CreditTermTemplatesViewer,
    CreditTermTemplatesWriter,
    CustomerViewer,
    CustomerWriter,
    CustodyViewer,
    CustodyWriter,
    DashboardViewer,
    DepositViewer,
    DepositWriter,
    DepositFreeze,
    DepositUnfreeze,
    ExposedConfigViewer,
    ExposedConfigWriter,
    GovernanceViewer,
    GovernanceWriter,
    ReportViewer,
    ReportWriter,
    AuditViewer,
}

impl std::str::FromStr for PermissionSetName {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PermissionSetName::*;
        match s {
            core_access::PERMISSION_SET_ACCESS_VIEWER => Ok(AccessViewer),
            core_access::PERMISSION_SET_ACCESS_WRITER => Ok(AccessWriter),

            core_accounting::PERMISSION_SET_ACCOUNTING_VIEWER => Ok(AccountingViewer),
            core_accounting::PERMISSION_SET_ACCOUNTING_WRITER => Ok(AccountingWriter),

            core_credit::PERMISSION_SET_CREDIT_VIEWER => Ok(CreditViewer),
            core_credit::PERMISSION_SET_CREDIT_WRITER => Ok(CreditWriter),
            core_credit::PERMISSION_SET_COLLECTION_VIEWER => Ok(CollectionViewer),
            core_credit::PERMISSION_SET_COLLECTION_WRITER => Ok(CollectionWriter),
            core_credit::PERMISSION_SET_COLLECTION_PAYMENT_DATE => Ok(CollectionPaymentDate),
            core_credit_terms::PERMISSION_SET_CREDIT_TERM_TEMPLATES_VIEWER => {
                Ok(CreditTermTemplatesViewer)
            }
            core_credit_terms::PERMISSION_SET_CREDIT_TERM_TEMPLATES_WRITER => {
                Ok(CreditTermTemplatesWriter)
            }

            core_customer::PERMISSION_SET_CUSTOMER_VIEWER => Ok(CustomerViewer),
            core_customer::PERMISSION_SET_CUSTOMER_WRITER => Ok(CustomerWriter),

            core_custody::PERMISSION_SET_CUSTODY_VIEWER => Ok(CustodyViewer),
            core_custody::PERMISSION_SET_CUSTODY_WRITER => Ok(CustodyWriter),

            dashboard::PERMISSION_SET_DASHBOARD_VIEWER => Ok(DashboardViewer),

            core_deposit::PERMISSION_SET_DEPOSIT_VIEWER => Ok(DepositViewer),
            core_deposit::PERMISSION_SET_DEPOSIT_WRITER => Ok(DepositWriter),
            core_deposit::PERMISSION_SET_DEPOSIT_FREEZE => Ok(DepositFreeze),
            core_deposit::PERMISSION_SET_DEPOSIT_UNFREEZE => Ok(DepositUnfreeze),

            domain_config::PERMISSION_SET_EXPOSED_CONFIG_VIEWER => Ok(ExposedConfigViewer),
            domain_config::PERMISSION_SET_EXPOSED_CONFIG_WRITER => Ok(ExposedConfigWriter),

            governance::PERMISSION_SET_GOVERNANCE_VIEWER => Ok(GovernanceViewer),
            governance::PERMISSION_SET_GOVERNANCE_WRITER => Ok(GovernanceWriter),

            core_report::PERMISSION_SET_REPORT_VIEWER => Ok(ReportViewer),
            core_report::PERMISSION_SET_REPORT_WRITER => Ok(ReportWriter),

            contract_creation::PERMISSION_SET_CONTRACT_CREATION => Ok(ContractCreation),

            PERMISSION_SET_AUDIT_VIEWER => Ok(AuditViewer),

            _ => Err(strum::ParseError::VariantNotFound),
        }
    }
}
