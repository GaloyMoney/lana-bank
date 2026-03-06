use std::{borrow::Cow, fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NoteTargetKind {
    Customer,
    Prospect,
    CreditFacility,
}

impl From<NoteTargetKind> for NoteTargetType {
    fn from(kind: NoteTargetKind) -> Self {
        NoteTargetType::new_from_string(kind.to_string())
    }
}

impl TryFrom<&NoteTargetType> for NoteTargetKind {
    type Error = strum::ParseError;

    fn try_from(t: &NoteTargetType) -> Result<Self, Self::Error> {
        t.as_str().parse()
    }
}

es_entity::entity_id! {
    NoteId
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct NoteTargetType(Cow<'static, str>);

impl NoteTargetType {
    pub const fn new(target: &'static str) -> Self {
        NoteTargetType(Cow::Borrowed(target))
    }

    pub fn new_from_string(target: String) -> Self {
        NoteTargetType(Cow::Owned(target))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for NoteTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type NoteAllOrOne = AllOrOne<NoteId>;

permission_sets_macro::permission_sets! {
    NoteWriter("Can create, update, and delete notes"),
    NoteReader("Can read notes"),
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum NoteObject {
    Note(NoteAllOrOne),
}

impl NoteObject {
    pub fn all_notes() -> NoteObject {
        NoteObject::Note(AllOrOne::All)
    }

    pub fn note(id: impl Into<Option<NoteId>>) -> NoteObject {
        match id.into() {
            Some(id) => NoteObject::Note(AllOrOne::ById(id)),
            None => NoteObject::all_notes(),
        }
    }
}

impl Display for NoteObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = NoteObjectDiscriminants::from(self);
        use NoteObject::*;
        match self {
            Note(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for NoteObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use NoteObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            Note => {
                let obj_ref = id.parse().map_err(|_| "could not parse NoteObject")?;
                NoteObject::Note(obj_ref)
            }
        };
        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreNoteAction {
    Note(NoteEntityAction),
}

impl CoreNoteAction {
    pub const NOTE_CREATE: Self = CoreNoteAction::Note(NoteEntityAction::Create);
    pub const NOTE_READ: Self = CoreNoteAction::Note(NoteEntityAction::Read);
    pub const NOTE_UPDATE: Self = CoreNoteAction::Note(NoteEntityAction::Update);
    pub const NOTE_DELETE: Self = CoreNoteAction::Note(NoteEntityAction::Delete);
    pub const NOTE_LIST: Self = CoreNoteAction::Note(NoteEntityAction::List);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreNoteActionDiscriminants::*;
        map_action!(note, Note, NoteEntityAction)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum NoteEntityAction {
    Create,
    Read,
    Update,
    Delete,
    List,
}

impl ActionPermission for NoteEntityAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Create | Self::Update | Self::Delete => PERMISSION_SET_NOTE_WRITER,
            Self::Read | Self::List => PERMISSION_SET_NOTE_READER,
        }
    }
}

impl Display for CoreNoteAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreNoteActionDiscriminants::from(self))?;
        use CoreNoteAction::*;
        match self {
            Note(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreNoteAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreNoteActionDiscriminants::*;
        let res = match entity.parse()? {
            Note => CoreNoteAction::from(action.parse::<NoteEntityAction>()?),
        };
        Ok(res)
    }
}

impl From<NoteEntityAction> for CoreNoteAction {
    fn from(action: NoteEntityAction) -> Self {
        CoreNoteAction::Note(action)
    }
}
