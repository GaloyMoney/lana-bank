use std::{fmt::Display, str::FromStr};

use authz::AllOrOne;
use chart_of_accounts::{new::CoreChartOfAccountsObjectNew, CoreChartOfAccountsObject};
use core_credit::CoreCreditObject;
use core_customer::{CustomerId, CustomerObject};
use core_user::UserObject;
use dashboard::DashboardModuleObject;
use deposit::CoreDepositObject;
use governance::GovernanceObject;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LanaObject {
    App(AppObject),
    Governance(GovernanceObject),
    User(UserObject),
    Customer(CustomerObject),
    ChartOfAccounts(CoreChartOfAccountsObject),
    NewChartOfAccounts(CoreChartOfAccountsObjectNew),
    Deposit(CoreDepositObject),
    Credit(CoreCreditObject),
    Dashboard(DashboardModuleObject),
}

impl From<AppObject> for LanaObject {
    fn from(object: AppObject) -> Self {
        LanaObject::App(object)
    }
}
impl From<DashboardModuleObject> for LanaObject {
    fn from(object: DashboardModuleObject) -> Self {
        LanaObject::Dashboard(object)
    }
}
impl From<GovernanceObject> for LanaObject {
    fn from(action: GovernanceObject) -> Self {
        LanaObject::Governance(action)
    }
}
impl From<UserObject> for LanaObject {
    fn from(action: UserObject) -> Self {
        LanaObject::User(action)
    }
}
impl From<CustomerObject> for LanaObject {
    fn from(action: CustomerObject) -> Self {
        LanaObject::Customer(action)
    }
}
impl From<CoreChartOfAccountsObject> for LanaObject {
    fn from(object: CoreChartOfAccountsObject) -> Self {
        LanaObject::ChartOfAccounts(object)
    }
}
impl From<CoreChartOfAccountsObjectNew> for LanaObject {
    fn from(object: CoreChartOfAccountsObjectNew) -> Self {
        LanaObject::NewChartOfAccounts(object)
    }
}
impl From<CoreDepositObject> for LanaObject {
    fn from(object: CoreDepositObject) -> Self {
        LanaObject::Deposit(object)
    }
}
impl From<CoreCreditObject> for LanaObject {
    fn from(object: CoreCreditObject) -> Self {
        LanaObject::Credit(object)
    }
}

impl Display for LanaObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", LanaObjectDiscriminants::from(self))?;
        use LanaObject::*;
        match self {
            App(action) => action.fmt(f),
            Governance(action) => action.fmt(f),
            User(action) => action.fmt(f),
            Customer(action) => action.fmt(f),
            ChartOfAccounts(action) => action.fmt(f),
            NewChartOfAccounts(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            Credit(action) => action.fmt(f),
            Dashboard(action) => action.fmt(f),
        }
    }
}

impl FromStr for LanaObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, object) = s.split_once('/').expect("missing colon");
        use LanaObjectDiscriminants::*;
        let res = match module.parse().expect("invalid module") {
            App => LanaObject::from(object.parse::<AppObject>()?),
            Governance => LanaObject::from(object.parse::<GovernanceObject>()?),
            User => LanaObject::from(object.parse::<UserObject>()?),
            Customer => LanaObject::from(object.parse::<CustomerObject>()?),
            ChartOfAccounts => LanaObject::from(object.parse::<CoreChartOfAccountsObject>()?),
            NewChartOfAccounts => LanaObject::from(object.parse::<CoreChartOfAccountsObjectNew>()?),
            Deposit => LanaObject::from(object.parse::<CoreDepositObject>()?),
            Credit => LanaObject::from(object.parse::<CoreCreditObject>()?),
            Dashboard => LanaObject::from(
                object
                    .parse::<DashboardModuleObject>()
                    .map_err(|_| "could not parse DashboardModuleObject")?,
            ),
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AppObject {
    Applicant,
    TermsTemplate,
    Document,
    Deposit,
    Withdrawal,
    Report,
    Audit,
    Ledger,
    TrialBalance,
    ProfitAndLossStatement,
    BalanceSheet,
    CashFlowStatement,
}

impl Display for AppObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", AppObjectDiscriminants::from(self))
    }
}

impl FromStr for AppObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split('/');
        let entity = elems.next().expect("missing first element");
        use AppObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Applicant => AppObject::Applicant,
            TermsTemplate => AppObject::TermsTemplate,
            Deposit => AppObject::Deposit,
            Withdrawal => AppObject::Withdrawal,
            Report => AppObject::Report,
            Audit => AppObject::Audit,
            Ledger => AppObject::Ledger,
            TrialBalance => AppObject::TrialBalance,
            ProfitAndLossStatement => AppObject::ProfitAndLossStatement,
            BalanceSheet => AppObject::BalanceSheet,
            CashFlowStatement => AppObject::CashFlowStatement,
            Document => AppObject::Document,
        };
        Ok(res)
    }
}

pub type CustomerAllOrOne = AllOrOne<CustomerId>;

#[cfg(test)]
mod test {
    use super::*;

    fn test_to_and_from_string(action: LanaObject, result: &str) -> anyhow::Result<()> {
        let action_str = action.to_string();
        assert_eq!(&action_str, result);

        let parsed_action: LanaObject = action_str.parse().expect("could not parse action");
        assert_eq!(parsed_action, action);

        Ok(())
    }

    #[test]
    fn action_serialization() -> anyhow::Result<()> {
        // App
        // test_to_and_from_string(
        //     LanaObject::App(AppObject::Customer(AllOrOne::All)),
        //     "app/customer/*",
        // )?;

        // Governance
        test_to_and_from_string(
            LanaObject::Governance(GovernanceObject::Committee(AllOrOne::All)),
            "governance/committee/*",
        )?;

        Ok(())
    }
}
