use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{action_description::*, AllOrOne};

es_entity::entity_id! {
    DocumentId;
}

pub type DocumentAllOrOne = AllOrOne<DocumentId>;

pub const PERMISSION_SET_DOCUMENT_STORAGE_VIEWER: &str = "document_storage_viewer";
pub const PERMISSION_SET_DOCUMENT_STORAGE_WRITER: &str = "document_storage_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum DocumentStorageObject {
    Document(DocumentAllOrOne),
}

impl DocumentStorageObject {
    pub fn all_documents() -> DocumentStorageObject {
        DocumentStorageObject::Document(AllOrOne::All)
    }
    pub fn document(id: impl Into<Option<DocumentId>>) -> DocumentStorageObject {
        match id.into() {
            Some(id) => DocumentStorageObject::Document(AllOrOne::ById(id)),
            None => DocumentStorageObject::all_documents(),
        }
    }
}

impl Display for DocumentStorageObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = DocumentStorageObjectDiscriminants::from(self);
        use DocumentStorageObject::*;
        match self {
            Document(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for DocumentStorageObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use DocumentStorageObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Document => {
                let obj_ref = id.parse().map_err(|_| "could not parse DocumentStorageObject")?;
                DocumentStorageObject::Document(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreDocumentStorageAction {
    Document(DocumentEntityAction),
}

impl CoreDocumentStorageAction {
    pub const DOCUMENT_CREATE: Self = CoreDocumentStorageAction::Document(DocumentEntityAction::Create);
    pub const DOCUMENT_READ: Self = CoreDocumentStorageAction::Document(DocumentEntityAction::Read);
    pub const DOCUMENT_LIST: Self = CoreDocumentStorageAction::Document(DocumentEntityAction::List);
    pub const DOCUMENT_UPLOAD: Self = CoreDocumentStorageAction::Document(DocumentEntityAction::Upload);

    pub fn entities() -> Vec<(
        CoreDocumentStorageActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreDocumentStorageActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreDocumentStorageActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                Document => DocumentEntityAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum DocumentEntityAction {
    Read,
    Create,
    List,
    Upload,
}

impl DocumentEntityAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_DOCUMENT_STORAGE_WRITER]),

                Self::Read => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_DOCUMENT_STORAGE_VIEWER,
                        PERMISSION_SET_DOCUMENT_STORAGE_WRITER,
                    ],
                ),

                Self::List => ActionDescription::new(
                    variant,
                    &[
                        PERMISSION_SET_DOCUMENT_STORAGE_WRITER,
                        PERMISSION_SET_DOCUMENT_STORAGE_VIEWER,
                    ],
                ),

                Self::Upload => ActionDescription::new(variant, &[PERMISSION_SET_DOCUMENT_STORAGE_WRITER]),
            };
            res.push(action_description);
        }

        res
    }
}

impl Display for CoreDocumentStorageAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreDocumentStorageActionDiscriminants::from(self))?;
        use CoreDocumentStorageAction::*;
        match self {
            Document(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreDocumentStorageAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreDocumentStorageActionDiscriminants::*;
        let res = match entity.parse()? {
            Document => CoreDocumentStorageAction::from(action.parse::<DocumentEntityAction>()?),
        };
        Ok(res)
    }
}

impl From<DocumentEntityAction> for CoreDocumentStorageAction {
    fn from(action: DocumentEntityAction) -> Self {
        CoreDocumentStorageAction::Document(action)
    }
}