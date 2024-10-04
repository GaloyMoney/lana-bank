use std::{fmt::Display, str::FromStr};

macro_rules! impl_trivial_action {
    ($from_type:ty, $variant:ident) => {
        impl From<$from_type> for Action {
            fn from(action: $from_type) -> Self {
                Action::$variant(action)
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Action {
    Loan(LoanAction),
    TermsTemplate(TermsTemplateAction),
    User(UserAction),
    Customer(CustomerAction),
    Deposit(DepositAction),
    Withdraw(WithdrawAction),
    Report(ReportAction),
    Audit(AuditAction),
    Ledger(LedgerAction),
    CreditFacility(CreditFacilityAction),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", ActionDiscriminants::from(self))?;
        use Action::*;
        match self {
            Loan(action) => action.fmt(f),
            TermsTemplate(action) => action.fmt(f),
            User(action) => action.fmt(f),
            Customer(action) => action.fmt(f),
            Deposit(action) => action.fmt(f),
            Withdraw(action) => action.fmt(f),
            Report(action) => action.fmt(f),
            Audit(action) => action.fmt(f),
            Ledger(action) => action.fmt(f),
            CreditFacility(action) => action.fmt(f),
        }
    }
}

impl FromStr for Action {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use ActionDiscriminants::*;
        let res = match entity.parse()? {
            Loan => Action::from(action.parse::<LoanAction>()?),
            TermsTemplate => Action::from(action.parse::<TermsTemplateAction>()?),
            User => Action::from(action.parse::<UserAction>()?),
            Customer => Action::from(action.parse::<CustomerAction>()?),
            Deposit => Action::from(action.parse::<DepositAction>()?),
            Withdraw => Action::from(action.parse::<WithdrawAction>()?),
            Report => Action::from(action.parse::<ReportAction>()?),
            Audit => Action::from(action.parse::<AuditAction>()?),
            Ledger => Action::from(action.parse::<LedgerAction>()?),
            CreditFacility => Action::from(action.parse::<CreditFacilityAction>()?),
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
    Approve,
    InitiateDisbursement,
    ApproveDisbursement,
    UpdateCollateral,
    RecordPayment,
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
pub enum UserAction {
    Read,
    Create,
    List,
    Update,
    Delete,
    AssignRole,
    RevokeRole,
}

impl_trivial_action!(UserAction, User);

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
pub enum WithdrawAction {
    Read,
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

    fn test_to_and_from_string(action: Action, result: &str) -> anyhow::Result<()> {
        let action_str = action.to_string();
        assert_eq!(&action_str, result);

        let parsed_action: Action = action_str.parse()?;
        assert_eq!(parsed_action, action);

        Ok(())
    }

    #[test]
    fn action_serialization() -> anyhow::Result<()> {
        // Loan
        test_to_and_from_string(Action::Loan(LoanAction::List), "loan:list")?;

        // UserAction
        test_to_and_from_string(Action::User(UserAction::AssignRole), "user:assign-role")?;
        test_to_and_from_string(Action::User(UserAction::Read), "user:read")?;
        Ok(())
    }
}
