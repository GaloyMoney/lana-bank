use authz::{AllOrOne, action_description::*};

es_entity::entity_id! {
    CustodianConfigId;
}

pub const PERMISSION_SET_CUSTODIAN_CONFIG_VIEWER: &str = "custody_viewer";
pub const PERMISSION_SET_CUSTODIAN_CONFIG_WRITER: &str = "custody_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyAction {
    CustodianConfig(CustodianConfigAction),
}

impl CoreCustodyAction {
    pub const CUSTODIAN_CONFIG_CREATE: Self =
        CoreCustodyAction::CustodianConfig(CustodianConfigAction::Create);

    pub fn entities() -> Vec<(
        CoreCustodyActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreCustodyActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreCustodyActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                CustodianConfig => CustodianConfigAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
    }
}

impl core::fmt::Display for CoreCustodyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCustodyActionDiscriminants::from(self))?;
        match self {
            Self::CustodianConfig(action) => action.fmt(f),
        }
    }
}

impl core::str::FromStr for CoreCustodyAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreCustodyActionDiscriminants::*;
        let res = match entity.parse()? {
            CustodianConfig => CoreCustodyAction::from(action.parse::<CustodianConfigAction>()?),
        };

        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CustodianConfigAction {
    Create,
    List,
}

impl CustodianConfigAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => {
                    ActionDescription::new(variant, &[PERMISSION_SET_CUSTODIAN_CONFIG_WRITER])
                }
                Self::List => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_CUSTODIAN_CONFIG_VIEWER,
                        PERMISSION_SET_CUSTODIAN_CONFIG_WRITER,
                    ],
                ),
            };
            res.push(action_description);
        }

        res
    }
}

impl From<CustodianConfigAction> for CoreCustodyAction {
    fn from(action: CustodianConfigAction) -> Self {
        Self::CustodianConfig(action)
    }
}

pub type CustodianConfigAllOrOne = AllOrOne<CustodianConfigId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyObject {
    CustodianConfig(CustodianConfigAllOrOne),
}

impl CoreCustodyObject {
    pub const fn all_custodian_configs() -> Self {
        CoreCustodyObject::CustodianConfig(AllOrOne::All)
    }
}

impl core::fmt::Display for CoreCustodyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCustodyObjectDiscriminants::from(self);
        match self {
            Self::CustodianConfig(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl core::str::FromStr for CoreCustodyObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCustodyObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            CustodianConfig => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCustodyObject")?;
                Self::CustodianConfig(obj_ref)
            }
        };
        Ok(res)
    }
}
