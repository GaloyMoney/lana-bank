use std::{fmt::Display, str::FromStr};

use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

es_entity::entity_id! {
    PdfGenerationId;

    PdfGenerationId => document_storage::DocumentId,
}

pub const PERMISSION_SET_PDF_GENERATION: &str = "pdf_generation";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum PdfGenerationModuleAction {
    Pdf(PdfAction),
}

impl PdfGenerationModuleAction {
    pub const PDF_CREATE: Self = PdfGenerationModuleAction::Pdf(PdfAction::Create);
    pub const PDF_FIND: Self = PdfGenerationModuleAction::Pdf(PdfAction::Find);
    pub const PDF_GENERATE_DOWNLOAD_LINK: Self =
        PdfGenerationModuleAction::Pdf(PdfAction::GenerateDownloadLink);

    pub fn actions() -> Vec<ActionMapping> {
        use PdfGenerationModuleActionDiscriminants::*;
        map_action!(pdf_generation, Pdf, PdfAction)
    }
}

impl Display for PdfGenerationModuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", PdfGenerationModuleActionDiscriminants::from(self))?;
        use PdfGenerationModuleAction::*;
        match self {
            Pdf(action) => action.fmt(f),
        }
    }
}

impl FromStr for PdfGenerationModuleAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use PdfGenerationModuleActionDiscriminants::*;
        let res = match entity.parse()? {
            Pdf => action.parse::<PdfAction>()?,
        };
        Ok(res.into())
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum PdfAction {
    Create,
    Find,
    GenerateDownloadLink,
}

impl ActionPermission for PdfAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Create | Self::Find | Self::GenerateDownloadLink => PERMISSION_SET_PDF_GENERATION,
        }
    }
}

pub type PdfAllOrOne = AllOrOne<PdfGenerationId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum PdfGenerationModuleObject {
    Pdf(PdfAllOrOne),
}

impl PdfGenerationModuleObject {
    pub const fn all_pdfs() -> Self {
        Self::Pdf(AllOrOne::All)
    }
}

impl From<PdfAction> for PdfGenerationModuleAction {
    fn from(action: PdfAction) -> Self {
        PdfGenerationModuleAction::Pdf(action)
    }
}

impl std::fmt::Display for PdfGenerationModuleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = PdfGenerationModuleObjectDiscriminants::from(self);
        match self {
            Self::Pdf(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for PdfGenerationModuleObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use PdfGenerationModuleObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Pdf => {
                let obj_ref = id.parse().map_err(|_| "could not parse PdfObject")?;
                PdfGenerationModuleObject::Pdf(obj_ref)
            }
        };
        Ok(res)
    }
}
