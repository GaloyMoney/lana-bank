use authz::{AllOrOne, action_description::*};

es_entity::entity_id! {
    CustodianId;
}

pub const PERMISSION_SET_CUSTODY_VIEWER: &str = "custody_viewer";
pub const PERMISSION_SET_CUSTODY_WRITER: &str = "custody_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyAction {
    Custodian(CustodianAction),
}

impl CoreCustodyAction {
    pub const CUSTODIAN_CREATE: Self = CoreCustodyAction::Custodian(CustodianAction::Create);
    pub const CUSTODIAN_LIST: Self = CoreCustodyAction::Custodian(CustodianAction::List);

    pub fn entities() -> Vec<(
        CoreCustodyActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreCustodyActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreCustodyActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                Custodian => CustodianAction::describe(),
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
            Self::Custodian(action) => action.fmt(f),
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
            Custodian => CoreCustodyAction::from(action.parse::<CustodianAction>()?),
        };

        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum CustodianAction {
    Create,
    List,
}

impl CustodianAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_CUSTODY_WRITER]),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_CUSTODY_VIEWER, PERMISSION_SET_CUSTODY_WRITER],
                ),
            };
            res.push(action_description);
        }

        res
    }
}

impl From<CustodianAction> for CoreCustodyAction {
    fn from(action: CustodianAction) -> Self {
        Self::Custodian(action)
    }
}

pub type CustodianAllOrOne = AllOrOne<CustodianId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCustodyObject {
    Custodian(CustodianAllOrOne),
}

impl CoreCustodyObject {
    pub const fn all_custodians() -> Self {
        CoreCustodyObject::Custodian(AllOrOne::All)
    }

    pub const fn custodian(id: CustodianId) -> Self {
        CoreCustodyObject::Custodian(AllOrOne::ById(id))
    }
}

impl core::fmt::Display for CoreCustodyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCustodyObjectDiscriminants::from(self);
        match self {
            Self::Custodian(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl core::str::FromStr for CoreCustodyObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCustodyObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Custodian => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCustodyObject")?;
                Self::Custodian(obj_ref)
            }
        };
        Ok(res)
    }
}
