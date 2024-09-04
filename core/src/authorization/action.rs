use std::{
    borrow::Cow,
    fmt::{self, Display},
    str::FromStr,
};

use crate::primitives::Role;

macro_rules! impl_from_for_action {
    ($from_type:ty, $variant:ident) => {
        impl From<$from_type> for Action {
            fn from(action: $from_type) -> Self {
                Action::$variant(action)
            }
        }
    };
}

macro_rules! impl_trivial_action {
    ($from_type:ty, $variant:ident) => {
        impl $from_type {
            fn add_to(&self, elems: &mut ActionElements<'_>) {
                elems.push_static(self)
            }
        }

        impl TryFrom<&[Cow<'_, str>]> for $from_type {
            type Error = strum::ParseError;

            fn try_from(elems: &[Cow<'_, str>]) -> Result<Self, Self::Error> {
                Self::from_str(elems[0].as_ref())
            }
        }

        impl From<$from_type> for Action {
            fn from(action: $from_type) -> Self {
                Action::$variant(action)
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::IntoStaticStr, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Action {
    Loan(LoanAction),
    Term(TermAction),
    User(UserAction),
    Customer(CustomerAction),
    Deposit(DepositAction),
    Withdraw(WithdrawAction),
    Audit(AuditAction),
    Ledger(LedgerAction),
}

impl From<Action> for ActionElements<'_> {
    fn from(action: Action) -> Self {
        let mut elems = ActionElements::empty();
        elems.push_static(ActionDiscriminants::from(action));
        use Action::*;
        match action {
            Loan(action) => action.add_to(&mut elems),
            Term(action) => action.add_to(&mut elems),
            User(action) => action.add_to(&mut elems),
            Customer(action) => action.add_to(&mut elems),
            Deposit(action) => action.add_to(&mut elems),
            Withdraw(action) => action.add_to(&mut elems),
            Audit(action) => action.add_to(&mut elems),
            Ledger(action) => action.add_to(&mut elems),
        }
        elems
    }
}

impl TryFrom<ActionElements<'_>> for Action {
    type Error = strum::ParseError;

    fn try_from(elems: ActionElements<'_>) -> Result<Self, Self::Error> {
        use ActionDiscriminants::*;
        let res = match ActionDiscriminants::from_str(elems.elems[0].as_ref())? {
            Loan => Action::from(LoanAction::try_from(&elems.elems[1..])?),
            Term => Action::from(TermAction::try_from(&elems.elems[1..])?),
            User => Action::from(UserAction::try_from(&elems.elems[1..])?),
            Customer => Action::from(CustomerAction::try_from(&elems.elems[1..])?),
            Deposit => Action::from(DepositAction::try_from(&elems.elems[1..])?),
            Withdraw => Action::from(WithdrawAction::try_from(&elems.elems[1..])?),
            Audit => Action::from(AuditAction::try_from(&elems.elems[1..])?),
            Ledger => Action::from(LedgerAction::try_from(&elems.elems[1..])?),
        };

        Ok(res)
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ActionElements::from(*self).fmt(f)
    }
}

impl std::str::FromStr for Action {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elems = ActionElements::from(s);
        Action::try_from(elems)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::IntoStaticStr, strum::EnumString)]
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

#[derive(PartialEq, Clone, Copy, Debug, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum TermAction {
    Read,
    Update,
}

impl_trivial_action!(TermAction, Term);

#[derive(Clone, PartialEq, Copy, Debug, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum AuditAction {
    List,
}

impl_trivial_action!(AuditAction, Audit);

#[derive(PartialEq, Clone, Copy, Debug, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::IntoStaticStr, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum UserAction {
    Read,
    Create,
    List,
    Update,
    Delete,
    AssignRole(Role),
    RevokeRole(Role),
}

impl UserAction {
    fn add_to(&self, elems: &mut ActionElements<'_>) {
        elems.push_static(UserActionDiscriminants::from(self));
        use UserAction::*;
        match self {
            Create | Read | List | Update | Delete => (),
            UserAction::AssignRole(role) | UserAction::RevokeRole(role) => elems.push_static(role),
        }
    }
}

impl TryFrom<&[Cow<'_, str>]> for UserAction {
    type Error = strum::ParseError;

    fn try_from(elems: &[Cow<'_, str>]) -> Result<Self, Self::Error> {
        use UserActionDiscriminants::*;
        let res = match UserActionDiscriminants::from_str(elems[0].as_ref())? {
            Read => UserAction::Read,
            Create => UserAction::Create,
            List => UserAction::List,
            Update => UserAction::Update,
            Delete => UserAction::Delete,
            AssignRole => UserAction::AssignRole(Role::from_str(elems[1].as_ref())?),
            RevokeRole => UserAction::RevokeRole(Role::from_str(elems[1].as_ref())?),
        };
        Ok(res)
    }
}

impl_from_for_action!(UserAction, User);

#[derive(PartialEq, Clone, Copy, Debug, strum::IntoStaticStr, strum::EnumString)]
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

#[derive(PartialEq, Clone, Copy, Debug, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum DepositAction {
    Read,
    Record,
    List,
}

impl_trivial_action!(DepositAction, Deposit);

#[derive(PartialEq, Clone, Copy, Debug, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum WithdrawAction {
    Read,
    Initiate,
    Confirm,
    List,
    Cancel,
}

impl_trivial_action!(WithdrawAction, Withdraw);

#[derive(PartialEq, Clone, Copy, Debug, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum LedgerAction {
    Read,
}

impl_trivial_action!(LedgerAction, Ledger);

struct ActionElements<'a> {
    elems: Vec<Cow<'a, str>>,
}

impl<'a> ActionElements<'a> {
    fn empty() -> Self {
        Self { elems: Vec::new() }
    }

    fn push_static(&mut self, elem: impl Into<&'static str>) {
        self.elems.push(Cow::Borrowed(elem.into()));
    }
}

impl<'a> Display for ActionElements<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, elem) in self.elems.iter().enumerate() {
            if i > 0 && i < self.elems.len() {
                write!(f, ":")?;
            }
            elem.fmt(f)?;
        }
        Ok(())
    }
}

impl<'a: 'b, 'b> From<&'a str> for ActionElements<'b> {
    fn from(s: &'a str) -> Self {
        let elems = s.split(':').map(Cow::Borrowed).collect();
        ActionElements { elems }
    }
}

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
        test_to_and_from_string(
            Action::User(UserAction::AssignRole(Role::Admin)),
            "user:assign-role:admin",
        )?;
        test_to_and_from_string(
            Action::User(UserAction::AssignRole(Role::BankManager)),
            "user:assign-role:bank-manager",
        )?;
        test_to_and_from_string(Action::User(UserAction::Read), "user:read")?;
        Ok(())
    }
}
