use authz::{ActionPermission, AllOrOne, action_description::*, map_action};
use std::str::FromStr;

use crate::TermsTemplateId;

pub type TermsTemplateAllOrOne = AllOrOne<TermsTemplateId>;

pub const PERMISSION_SET_TERMS_VIEWER: &str = "terms_viewer";
pub const PERMISSION_SET_CREDIT_TERM_TEMPLATES: &str = "credit_term_templates";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreTermsObject {
    TermsTemplate(TermsTemplateAllOrOne),
}

impl CoreTermsObject {
    pub fn terms_template(id: TermsTemplateId) -> Self {
        CoreTermsObject::TermsTemplate(AllOrOne::ById(id))
    }

    pub fn all_terms_templates() -> Self {
        CoreTermsObject::TermsTemplate(AllOrOne::All)
    }
}

impl std::fmt::Display for CoreTermsObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreTermsObjectDiscriminants::from(self);
        use CoreTermsObject::*;
        match self {
            TermsTemplate(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreTermsObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreTermsObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            TermsTemplate => {
                let obj_ref = id.parse().map_err(|_| "could not parse CoreTermsObject")?;
                CoreTermsObject::TermsTemplate(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreTermsAction {
    TermsTemplate(TermsTemplateAction),
}

impl CoreTermsAction {
    pub const TERMS_TEMPLATE_CREATE: Self =
        CoreTermsAction::TermsTemplate(TermsTemplateAction::Create);
    pub const TERMS_TEMPLATE_READ: Self = CoreTermsAction::TermsTemplate(TermsTemplateAction::Read);
    pub const TERMS_TEMPLATE_UPDATE: Self =
        CoreTermsAction::TermsTemplate(TermsTemplateAction::Update);
    pub const TERMS_TEMPLATE_LIST: Self = CoreTermsAction::TermsTemplate(TermsTemplateAction::List);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreTermsActionDiscriminants::*;
        use strum::VariantArray;

        CoreTermsActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                TermsTemplate => map_action!(terms, TermsTemplate, TermsTemplateAction),
            })
            .collect()
    }
}

impl std::fmt::Display for CoreTermsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreTermsActionDiscriminants::from(self))?;
        use CoreTermsAction::*;
        match self {
            TermsTemplate(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreTermsAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split(':');
        let entity = elems.next().expect("missing first element");
        let action = elems.next().expect("missing second element");
        use CoreTermsActionDiscriminants::*;
        let res = match entity.parse()? {
            TermsTemplate => CoreTermsAction::from(action.parse::<TermsTemplateAction>()?),
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
            Self::Read | Self::List => PERMISSION_SET_TERMS_VIEWER,
            Self::Create | Self::Update => PERMISSION_SET_CREDIT_TERM_TEMPLATES,
        }
    }
}

impl From<TermsTemplateAction> for CoreTermsAction {
    fn from(action: TermsTemplateAction) -> Self {
        Self::TermsTemplate(action)
    }
}

pub struct TermsPermissions;

impl crate::TermsTemplatePermissions for TermsPermissions {
    type Action = CoreTermsAction;
    type Object = CoreTermsObject;

    fn terms_template_create_action() -> Self::Action {
        CoreTermsAction::TERMS_TEMPLATE_CREATE
    }

    fn terms_template_read_action() -> Self::Action {
        CoreTermsAction::TERMS_TEMPLATE_READ
    }

    fn terms_template_update_action() -> Self::Action {
        CoreTermsAction::TERMS_TEMPLATE_UPDATE
    }

    fn terms_template_list_action() -> Self::Action {
        CoreTermsAction::TERMS_TEMPLATE_LIST
    }

    fn all_terms_templates_object() -> Self::Object {
        CoreTermsObject::all_terms_templates()
    }

    fn terms_template_object(id: TermsTemplateId) -> Self::Object {
        CoreTermsObject::terms_template(id)
    }
}
