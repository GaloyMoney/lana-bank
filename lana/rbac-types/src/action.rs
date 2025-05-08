use std::{fmt::Display, str::FromStr};

use core_accounting::CoreAccountingAction;
use core_credit::CoreCreditAction;
use core_customer::CoreCustomerAction;
use core_user::CoreUserAction;
use dashboard::DashboardModuleAction;
use deposit::CoreDepositAction;
use governance::GovernanceAction;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LanaAction {
    App(AppAction),
    Governance(GovernanceAction),
    User(CoreUserAction),
    Customer(CoreCustomerAction),
    Accounting(CoreAccountingAction),
    Dashboard(DashboardModuleAction),
    Deposit(CoreDepositAction),
    Credit(CoreCreditAction),
}

impl From<AppAction> for LanaAction {
    fn from(action: AppAction) -> Self {
        LanaAction::App(action)
    }
}
impl From<DashboardModuleAction> for LanaAction {
    fn from(action: DashboardModuleAction) -> Self {
        LanaAction::Dashboard(action)
    }
}
impl From<GovernanceAction> for LanaAction {
    fn from(action: GovernanceAction) -> Self {
        LanaAction::Governance(action)
    }
}
impl From<CoreUserAction> for LanaAction {
    fn from(action: CoreUserAction) -> Self {
        LanaAction::User(action)
    }
}
impl From<CoreCustomerAction> for LanaAction {
    fn from(action: CoreCustomerAction) -> Self {
        LanaAction::Customer(action)
    }
}
impl From<CoreAccountingAction> for LanaAction {
    fn from(action: CoreAccountingAction) -> Self {
        LanaAction::Accounting(action)
    }
}
impl From<CoreDepositAction> for LanaAction {
    fn from(action: CoreDepositAction) -> Self {
        LanaAction::Deposit(action)
    }
}
impl From<CoreCreditAction> for LanaAction {
    fn from(action: CoreCreditAction) -> Self {
        LanaAction::Credit(action)
    }
}

impl Display for LanaAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", LanaActionDiscriminants::from(self))?;
        use LanaAction::*;
        match self {
            App(action) => action.fmt(f),
            Governance(action) => action.fmt(f),
            User(action) => action.fmt(f),
            Customer(action) => action.fmt(f),
            Dashboard(action) => action.fmt(f),
            Accounting(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            Credit(action) => action.fmt(f),
        }
    }
}

impl FromStr for LanaAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, action) = s.split_once(':').expect("missing colon");
        use LanaActionDiscriminants::*;
        let res = match module.parse()? {
            App => LanaAction::from(action.parse::<AppAction>()?),
            Governance => LanaAction::from(action.parse::<GovernanceAction>()?),
            User => LanaAction::from(action.parse::<CoreUserAction>()?),
            Customer => LanaAction::from(action.parse::<CoreCustomerAction>()?),
            Dashboard => LanaAction::from(action.parse::<DashboardModuleAction>()?),
            Accounting => LanaAction::from(action.parse::<CoreAccountingAction>()?),
            Deposit => LanaAction::from(action.parse::<CoreDepositAction>()?),
            Credit => LanaAction::from(action.parse::<CoreCreditAction>()?),
        };
        Ok(res)
    }
}

macro_rules! impl_trivial_action {
    ($from_type:ty, $variant:ident) => {
        impl From<$from_type> for AppAction {
            fn from(action: $from_type) -> Self {
                AppAction::$variant(action)
            }
        }

        impl From<$from_type> for LanaAction {
            fn from(action: $from_type) -> Self {
                LanaAction::App(AppAction::$variant(action))
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AppAction {
    TermsTemplate(TermsTemplateAction),
    Report(ReportAction),
    Audit(AuditAction),
    Document(DocumentAction),
}

impl Display for AppAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", AppActionDiscriminants::from(self))?;
        use AppAction::*;
        match self {
            TermsTemplate(action) => action.fmt(f),
            Report(action) => action.fmt(f),
            Audit(action) => action.fmt(f),
            Document(action) => action.fmt(f),
        }
    }
}

impl FromStr for AppAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use AppActionDiscriminants::*;
        let res = match entity.parse()? {
            TermsTemplate => AppAction::from(action.parse::<TermsTemplateAction>()?),
            Report => AppAction::from(action.parse::<ReportAction>()?),
            Audit => AppAction::from(action.parse::<AuditAction>()?),
            Document => AppAction::from(action.parse::<DocumentAction>()?),
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum TermsTemplateAction {
    Read,
    Update,
    Create,
    List,
}

impl_trivial_action!(TermsTemplateAction, TermsTemplate);

#[derive(Clone, PartialEq, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum AuditAction {
    List,
}

impl_trivial_action!(AuditAction, Audit);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum DocumentAction {
    Create,
    Read,
    List,
    GenerateDownloadLink,
    Delete,
    Archive,
}

impl_trivial_action!(DocumentAction, Document);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportAction {
    Read,
    List,
    Create,
    Upload,
    GenerateDownloadLink,
}

impl_trivial_action!(ReportAction, Report);

#[cfg(test)]
mod test {
    use super::*;

    fn test_to_and_from_string(action: LanaAction, result: &str) -> anyhow::Result<()> {
        let action_str = action.to_string();
        assert_eq!(&action_str, result);

        let parsed_action: LanaAction = action_str.parse()?;
        assert_eq!(parsed_action, action);

        Ok(())
    }

    #[test]
    fn action_serialization() -> anyhow::Result<()> {
        // Report
        test_to_and_from_string(
            LanaAction::App(AppAction::Report(ReportAction::List)),
            "app:report:list",
        )?;
        Ok(())
    }
}
