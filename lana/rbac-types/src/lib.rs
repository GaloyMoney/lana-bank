#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod action;
mod audit_action;
mod audit_object;
mod object;

use serde::{Deserialize, Serialize};
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

/// Macro to define a permission set enum with automatic FromStr/Display implementations.
///
/// This macro generates:
/// - An enum with the specified variants
/// - A FromStr implementation that maps string constants to enum variants
/// - A Display implementation that maps enum variants back to strings
/// - All the derives needed for GraphQL and testing
///
/// # Example
/// ```ignore
/// permission_set_enum! {
///     PermissionSetName {
///         AccessViewer = core_access::PERMISSION_SET_ACCESS_VIEWER,
///         AccessWriter = core_access::PERMISSION_SET_ACCESS_WRITER,
///     }
/// }
/// ```
macro_rules! permission_set_enum {
    (
        $enum_name:ident {
            $( $variant:ident = $const_path:expr ),* $(,)?
        }
    ) => {
        #[derive(Clone, PartialEq, Eq, Copy, Debug, async_graphql::Enum, strum::VariantArray)]
        pub enum $enum_name {
            $( $variant, )*
        }

        impl std::str::FromStr for $enum_name {
            type Err = strum::ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if s == $const_path {
                        return Ok(Self::$variant);
                    }
                )*
                Err(strum::ParseError::VariantNotFound)
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $(
                    if matches!(self, Self::$variant) {
                        return write!(f, "{}", $const_path);
                    }
                )*
                unreachable!("All variants should be covered")
            }
        }
    };
}

permission_set_enum! {
    PermissionSetName {
        AccessViewer = core_access::PERMISSION_SET_ACCESS_VIEWER,
        AccessWriter = core_access::PERMISSION_SET_ACCESS_WRITER,
        AccountingViewer = core_accounting::PERMISSION_SET_ACCOUNTING_VIEWER,
        AccountingWriter = core_accounting::PERMISSION_SET_ACCOUNTING_WRITER,
        CollectionViewer = core_credit::PERMISSION_SET_COLLECTION_VIEWER,
        CollectionWriter = core_credit::PERMISSION_SET_COLLECTION_WRITER,
        CollectionPaymentDate = core_credit::PERMISSION_SET_COLLECTION_PAYMENT_DATE,
        ContractCreation = contract_creation::PERMISSION_SET_CONTRACT_CREATION,
        CreditViewer = core_credit::PERMISSION_SET_CREDIT_VIEWER,
        CreditWriter = core_credit::PERMISSION_SET_CREDIT_WRITER,
        CreditTermTemplatesViewer = core_credit_terms::PERMISSION_SET_CREDIT_TERM_TEMPLATES_VIEWER,
        CreditTermTemplatesWriter = core_credit_terms::PERMISSION_SET_CREDIT_TERM_TEMPLATES_WRITER,
        CustomerViewer = core_customer::PERMISSION_SET_CUSTOMER_VIEWER,
        CustomerWriter = core_customer::PERMISSION_SET_CUSTOMER_WRITER,
        CustodyViewer = core_custody::PERMISSION_SET_CUSTODY_VIEWER,
        CustodyWriter = core_custody::PERMISSION_SET_CUSTODY_WRITER,
        DashboardViewer = dashboard::PERMISSION_SET_DASHBOARD_VIEWER,
        DepositViewer = core_deposit::PERMISSION_SET_DEPOSIT_VIEWER,
        DepositWriter = core_deposit::PERMISSION_SET_DEPOSIT_WRITER,
        DepositFreeze = core_deposit::PERMISSION_SET_DEPOSIT_FREEZE,
        DepositUnfreeze = core_deposit::PERMISSION_SET_DEPOSIT_UNFREEZE,
        ExposedConfigViewer = domain_config::PERMISSION_SET_EXPOSED_CONFIG_VIEWER,
        ExposedConfigWriter = domain_config::PERMISSION_SET_EXPOSED_CONFIG_WRITER,
        GovernanceViewer = governance::PERMISSION_SET_GOVERNANCE_VIEWER,
        GovernanceWriter = governance::PERMISSION_SET_GOVERNANCE_WRITER,
        ReportViewer = core_report::PERMISSION_SET_REPORT_VIEWER,
        ReportWriter = core_report::PERMISSION_SET_REPORT_WRITER,
        AuditViewer = PERMISSION_SET_AUDIT_VIEWER,
    }
}

#[derive(Clone, Copy, Debug, strum::EnumDiscriminants, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use strum::VariantArray;

    /// This test ensures that all PermissionSetName variants can round-trip
    /// through Display and FromStr.
    ///
    /// The macro generates both implementations, so this verifies they're consistent.
    #[test]
    fn permission_set_name_round_trip() {
        // Test all variants can round-trip
        for variant in PermissionSetName::VARIANTS {
            let as_string = variant.to_string();
            let parsed: PermissionSetName = as_string
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse '{}' for {:?}", as_string, variant));
            assert_eq!(
                &parsed, variant,
                "Round-trip failed for {:?}: {} -> {:?}",
                variant, as_string, parsed
            );
        }
    }

    #[test]
    fn permission_set_name_parse_invalid() {
        let result = "invalid_permission_set".parse::<PermissionSetName>();
        assert!(result.is_err(), "Should fail to parse invalid permission set");
    }
}
