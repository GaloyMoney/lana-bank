use authz::{ActionPermission, AllOrOne, action_description::*, map_action};
use std::str::FromStr;

use crate::TermsTemplateId;

pub type TermsTemplateAllOrOne = AllOrOne<TermsTemplateId>;

pub const PERMISSION_SET_CREDIT_TERM_TEMPLATES_VIEWER: &str = "credit_term_templates_viewer";
pub const PERMISSION_SET_CREDIT_TERM_TEMPLATES_WRITER: &str = "credit_term_templates_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditTermsObject {
    TermsTemplate(TermsTemplateAllOrOne),
}

impl CoreCreditTermsObject {
    pub fn terms_template(id: TermsTemplateId) -> Self {
        CoreCreditTermsObject::TermsTemplate(AllOrOne::ById(id))
    }

    pub fn all_terms_templates() -> Self {
        CoreCreditTermsObject::TermsTemplate(AllOrOne::All)
    }
}

impl std::fmt::Display for CoreCreditTermsObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreCreditTermsObjectDiscriminants::from(self);
        use CoreCreditTermsObject::*;
        match self {
            TermsTemplate(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreCreditTermsObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreCreditTermsObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            TermsTemplate => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse CoreCreditTermsObject")?;
                CoreCreditTermsObject::TermsTemplate(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreCreditTermsAction {
    TermsTemplate(TermsTemplateAction),
}

impl CoreCreditTermsAction {
    pub const TERMS_TEMPLATE_CREATE: Self =
        CoreCreditTermsAction::TermsTemplate(TermsTemplateAction::Create);
    pub const TERMS_TEMPLATE_READ: Self =
        CoreCreditTermsAction::TermsTemplate(TermsTemplateAction::Read);
    pub const TERMS_TEMPLATE_UPDATE: Self =
        CoreCreditTermsAction::TermsTemplate(TermsTemplateAction::Update);
    pub const TERMS_TEMPLATE_LIST: Self =
        CoreCreditTermsAction::TermsTemplate(TermsTemplateAction::List);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreCreditTermsActionDiscriminants::*;
        use strum::VariantArray;

        CoreCreditTermsActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                TermsTemplate => map_action!(terms, TermsTemplate, TermsTemplateAction),
            })
            .collect()
    }
}

impl std::fmt::Display for CoreCreditTermsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreCreditTermsActionDiscriminants::from(self))?;
        use CoreCreditTermsAction::*;
        match self {
            TermsTemplate(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreCreditTermsAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreCreditTermsActionDiscriminants::*;
        let res = match entity.parse()? {
            TermsTemplate => CoreCreditTermsAction::from(action.parse::<TermsTemplateAction>()?),
        };
        Ok(res)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum TermsTemplateAction {
    Create,
    Read,
    Update,
    List,
}

impl ActionPermission for TermsTemplateAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_CREDIT_TERM_TEMPLATES_VIEWER,
            Self::Create | Self::Update => PERMISSION_SET_CREDIT_TERM_TEMPLATES_WRITER,
        }
    }
}

impl From<TermsTemplateAction> for CoreCreditTermsAction {
    fn from(action: TermsTemplateAction) -> Self {
        Self::TermsTemplate(action)
    }
}
