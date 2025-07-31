use audit::AuditId;
use authz::AllOrOne;
use std::{fmt::Display, str::FromStr};

pub type AuditAllOrOne = AllOrOne<AuditId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum AuditObject {
    Audit(AuditAllOrOne),
}

impl AuditObject {
    pub const fn all_audits() -> Self {
        Self::Audit(AllOrOne::All)
    }
}

impl Display for AuditObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = AuditObjectDiscriminants::from(self);
        match self {
            Self::Audit(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for AuditObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use AuditObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Audit => {
                let obj_ref = id.parse().map_err(|_| "could not parse Audit")?;
                Self::Audit(obj_ref)
            }
        };

        Ok(res)
    }
}
