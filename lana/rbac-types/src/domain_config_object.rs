use authz::AllOrOne;
use domain_config::DomainConfigId;
use std::{fmt::Display, str::FromStr};

pub type ExposedConfigAllOrOne = AllOrOne<DomainConfigId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DomainConfigObject {
    ExposedConfig(ExposedConfigAllOrOne),
}

impl DomainConfigObject {
    pub const fn all_exposed_configs() -> Self {
        Self::ExposedConfig(AllOrOne::All)
    }
}

impl Display for DomainConfigObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = DomainConfigObjectDiscriminants::from(self);
        match self {
            Self::ExposedConfig(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for DomainConfigObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use DomainConfigObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            ExposedConfig => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse DomainConfigObject")?;
                Self::ExposedConfig(obj_ref)
            }
        };
        Ok(res)
    }
}
