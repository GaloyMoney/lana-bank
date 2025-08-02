use authz::{ActionPermission, action_description::*, auto_mappings};
use std::{fmt::Display, str::FromStr};

pub const PERMISSION_SET_AUDIT_VIEWER: &str = "audit_viewer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AuditAction {
    Audit(AuditEntityAction),
}

impl AuditAction {
    pub fn actions() -> Vec<ActionDescription> {
        use AuditActionDiscriminants::*;
        auto_mappings!(Audit => AuditEntityAction)
    }
}

impl Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", AuditActionDiscriminants::from(self))?;
        use AuditAction::*;
        match self {
            Audit(action) => action.fmt(f),
        }
    }
}

impl FromStr for AuditAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use AuditActionDiscriminants::*;
        let res = match entity.parse()? {
            Audit => AuditAction::from(action.parse::<AuditEntityAction>()?),
        };
        Ok(res)
    }
}

#[derive(Clone, PartialEq, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum AuditEntityAction {
    List,
}

impl ActionPermission for AuditEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List => PERMISSION_SET_AUDIT_VIEWER,
        }
    }
}

impl From<AuditEntityAction> for AuditAction {
    fn from(action: AuditEntityAction) -> Self {
        AuditAction::Audit(action)
    }
}
