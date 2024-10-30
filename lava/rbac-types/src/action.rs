use std::{fmt::Display, str::FromStr};

use core_user::CoreUserAction;
use governance::GovernanceAction;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LavaAction {
    App(AppAction),
    Governance(GovernanceAction),
    User(CoreUserAction),
}

impl From<AppAction> for LavaAction {
    fn from(action: AppAction) -> Self {
        LavaAction::App(action)
    }
}
impl From<GovernanceAction> for LavaAction {
    fn from(action: GovernanceAction) -> Self {
        LavaAction::Governance(action)
    }
}
impl From<CoreUserAction> for LavaAction {
    fn from(action: CoreUserAction) -> Self {
        LavaAction::User(action)
    }
}

impl Display for LavaAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", LavaActionDiscriminants::from(self))?;
        use LavaAction::*;
        match self {
            App(action) => action.fmt(f),
            Governance(action) => action.fmt(f),
            User(action) => action.fmt(f),
        }
    }
}

impl FromStr for LavaAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, action) = s.split_once(':').expect("missing colon");
        use LavaActionDiscriminants::*;
        let res = match module.parse()? {
            App => LavaAction::from(action.parse::<AppAction>()?),
            Governance => LavaAction::from(action.parse::<GovernanceAction>()?),
            User => LavaAction::from(action.parse::<CoreUserAction>()?),
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

        impl From<$from_type> for LavaAction {
            fn from(action: $from_type) -> Self {
                LavaAction::App(AppAction::$variant(action))
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AppAction {
    Loan(LoanAction),
    TermsTemplate(TermsTemplateAction),
    Customer(CustomerAction),
    Deposit(DepositAction),
    Withdraw(WithdrawAction),
    Report(ReportAction),
    Audit(AuditAction),
    Ledger(LedgerAction),
    CreditFacility(CreditFacilityAction),
    Document(DocumentAction),
}

impl Display for AppAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", AppActionDiscriminants::from(self))?;
        use AppAction::*;
        match self {
            Loan(action) => action.fmt(f),
            TermsTemplate(action) => action.fmt(f),
            Customer(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            Withdraw(action) => action.fmt(f),
            Report(action) => action.fmt(f),
            Audit(action) => action.fmt(f),
            Ledger(action) => action.fmt(f),
            CreditFacility(action) => action.fmt(f),
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
            Loan => AppAction::from(action.parse::<LoanAction>()?),
            TermsTemplate => AppAction::from(action.parse::<TermsTemplateAction>()?),
            Customer => AppAction::from(action.parse::<CustomerAction>()?),
            Deposit => AppAction::from(action.parse::<DepositAction>()?),
            Withdraw => AppAction::from(action.parse::<WithdrawAction>()?),
            Report => AppAction::from(action.parse::<ReportAction>()?),
            Audit => AppAction::from(action.parse::<AuditAction>()?),
            Ledger => AppAction::from(action.parse::<LedgerAction>()?),
            CreditFacility => AppAction::from(action.parse::<CreditFacilityAction>()?),
            Document => AppAction::from(action.parse::<DocumentAction>()?),
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum LoanAction {
    Read,
    List,
    Create,
    Approve,
    RecordPayment,
    UpdateCollateral,
    RecordInterest,
    UpdateCollateralizationState,
}

impl_trivial_action!(LoanAction, Loan);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CreditFacilityAction {
    Create,
    Read,
    List,
    ConcludeApprovalProcess,
    Activate,
    InitiateDisbursement,
    ConfirmDisbursement,
    ListDisbursement,
    UpdateCollateral,
    RecordPayment,
    RecordInterest,
    Complete,
    UpdateCollateralizationState,
}

impl_trivial_action!(CreditFacilityAction, CreditFacility);

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
pub enum CustomerAction {
    Read,
    Create,
    StartKyc,
    ApproveKyc,
    DeclineKyc,
    List,
    Update,
}

impl_trivial_action!(CustomerAction, Customer);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum DepositAction {
    Read,
    Record,
    List,
}

impl_trivial_action!(DepositAction, Deposit);

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
pub enum WithdrawAction {
    Read,
    ConcludeApprovalProcess,
    Initiate,
    Confirm,
    List,
    Cancel,
}

impl_trivial_action!(WithdrawAction, Withdraw);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportAction {
    Read,
    List,
    Create,
    Compile,
    Invoke,
    Upload,
    GenerateDownloadLink,
}

impl_trivial_action!(ReportAction, Report);

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum LedgerAction {
    Read,
}

impl_trivial_action!(LedgerAction, Ledger);

#[cfg(test)]
mod test {
    use super::*;

    fn test_to_and_from_string(action: LavaAction, result: &str) -> anyhow::Result<()> {
        let action_str = action.to_string();
        assert_eq!(&action_str, result);

        let parsed_action: LavaAction = action_str.parse()?;
        assert_eq!(parsed_action, action);

        Ok(())
    }

    #[test]
    fn action_serialization() -> anyhow::Result<()> {
        // Loan
        test_to_and_from_string(
            LavaAction::App(AppAction::Loan(LoanAction::List)),
            "app:loan:list",
        )?;
        Ok(())
    }
}
