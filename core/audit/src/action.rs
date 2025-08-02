use authz::action_description::*;
use std::{fmt::Display, str::FromStr};

pub const PERMISSION_SET_AUDIT_VIEWER: &str = "audit_viewer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AuditAction {
    Audit(AuditEntityAction),
}

impl AuditAction {
    pub fn entities() -> Vec<(AuditActionDiscriminants, Vec<ActionDescription<NoPath>>)> {
        use AuditActionDiscriminants::*;

        let mut result = vec![];

        for entity in <AuditActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                Audit => AuditEntityAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
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

impl AuditEntityAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::List => ActionDescription::new(variant, &[PERMISSION_SET_AUDIT_VIEWER]),
            };
            res.push(action_description);
        }

        res
    }
}

impl From<AuditEntityAction> for AuditAction {
    fn from(action: AuditEntityAction) -> Self {
        AuditAction::Audit(action)
    }
}
