#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod action;
mod audit_action;
mod audit_object;
mod object;

use tracing::Level;
use tracing_utils::ErrorSeverity;
use uuid::{Uuid, uuid};

use core_access::UserId;
use core_customer::CustomerId;

pub use action::*;
pub use audit_action::*;
pub use audit_object::*;
pub use object::*;

const SYSTEM_SUBJECT_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
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

#[derive(Clone, Copy, Debug, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::AsRefStr, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Subject {
    Customer(CustomerId),
    User(UserId),
    System,
}

impl audit::SystemSubject for Subject {
    fn system() -> Self {
        Subject::System
    }
}

impl std::str::FromStr for Subject {
    type Err = ParseSubjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ParseSubjectError::InvalidSubjectFormat);
        }

        let id: uuid::Uuid = parts[1].parse()?;
        use SubjectDiscriminants::*;
        let res = match SubjectDiscriminants::from_str(parts[0])? {
            Customer => Subject::Customer(CustomerId::from(id)),
            User => Subject::User(UserId::from(id)),
            System => Subject::System,
        };
        Ok(res)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseSubjectError {
    #[error("ParseSubjectError - Strum: {0}")]
    Strum(#[from] strum::ParseError),
    #[error("ParseSubjectError - Uuid: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("ParseSubjectError - InvalidSubjectFormat")]
    InvalidSubjectFormat,
}

impl ErrorSeverity for ParseSubjectError {
    fn severity(&self) -> Level {
        match self {
            Self::Strum(_) => Level::WARN,
            Self::Uuid(_) => Level::WARN,
            Self::InvalidSubjectFormat => Level::WARN,
        }
    }
}

impl From<UserId> for Subject {
    fn from(s: UserId) -> Self {
        Subject::User(s)
    }
}

impl From<CustomerId> for Subject {
    fn from(s: CustomerId) -> Self {
        Subject::Customer(s)
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id: uuid::Uuid = match self {
            Subject::Customer(id) => id.into(),
            Subject::User(id) => id.into(),
            Subject::System => SYSTEM_SUBJECT_ID,
        };
        write!(f, "{}:{}", SubjectDiscriminants::from(self).as_ref(), id)?;
        Ok(())
    }
}

impl TryFrom<&Subject> for core_deposit::DepositAccountHolderId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::Customer(id) => Ok(core_deposit::DepositAccountHolderId::from(*id)),
            _ => Err("Subject is not Customer"),
        }
    }
}

impl TryFrom<&Subject> for CustomerId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::Customer(id) => Ok(*id),
            _ => Err("Subject is not Customer"),
        }
    }
}

impl TryFrom<&Subject> for UserId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::User(id) => Ok(*id),
            _ => Err("Subject is not User"),
        }
    }
}

impl TryFrom<&Subject> for governance::CommitteeMemberId {
    type Error = &'static str;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        match value {
            Subject::User(id) => Ok(Self::from(*id)),
            _ => Err("Subject is not User"),
        }
    }
}
