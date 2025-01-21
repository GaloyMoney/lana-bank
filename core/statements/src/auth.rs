use std::{fmt::Display, str::FromStr};

use authz::AllOrOne;

use super::primitives::StatementId;

pub type StatementAllOrOne = AllOrOne<StatementId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreStatementsAction {
    StatementAction(StatementAction),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreStatementsObject {
    Statement(StatementAllOrOne),
}

impl CoreStatementsObject {
    pub fn statement(id: StatementId) -> Self {
        CoreStatementsObject::Statement(AllOrOne::ById(id))
    }

    pub fn all_statements() -> Self {
        CoreStatementsObject::Statement(AllOrOne::All)
    }
}

impl Display for CoreStatementsObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreStatementsObjectDiscriminants::from(self);
        use CoreStatementsObject::*;
        match self {
            Statement(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CoreStatementsObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreStatementsObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Statement => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreStatementObject")?;
                CoreStatementsObject::Statement(obj_ref)
            }
        };
        Ok(res)
    }
}

impl CoreStatementsAction {
    pub const STATEMENT_CREATE: Self =
        CoreStatementsAction::StatementAction(StatementAction::Create);
    pub const STATEMENT_READ: Self = CoreStatementsAction::StatementAction(StatementAction::Read);
}

impl Display for CoreStatementsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreStatementsActionDiscriminants::from(self))?;
        use CoreStatementsAction::*;
        match self {
            StatementAction(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreStatementsAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        let res = match entity.parse()? {
            CoreStatementsActionDiscriminants::StatementAction => {
                CoreStatementsAction::from(action.parse::<StatementAction>()?)
            }
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum StatementAction {
    Create,
    Read,
}

impl From<StatementAction> for CoreStatementsAction {
    fn from(action: StatementAction) -> Self {
        CoreStatementsAction::StatementAction(action)
    }
}
