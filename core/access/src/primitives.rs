use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{ActionPermission, AllOrOne, action_description::*, map_action};

#[cfg(feature = "governance")]
es_entity::entity_id! {
    UserId;
    UserId => governance::CommitteeMemberId,
}
#[cfg(not(feature = "governance"))]
es_entity::entity_id! { UserId }

es_entity::entity_id! { PermissionSetId, RoleId }

pub const ROLE_NAME_SUPERUSER: &str = "superuser";

pub const PERMISSION_SET_ACCESS_WRITER: &str = "access_writer";
pub const PERMISSION_SET_ACCESS_VIEWER: &str = "access_viewer";

/// Type representing a role identifier for an underlying authorization subsystem.
/// Any type that is convertible to `AuthRoleToken` can be used as such role.
#[derive(Clone, Debug)]
pub struct AuthRoleToken {
    prefix: &'static str,
    id: String,
}

impl AuthRoleToken {
    pub fn new<Id: Display>(prefix: &'static str, id: Id) -> Self {
        Self {
            prefix,
            id: id.to_string(),
        }
    }
}

impl From<RoleId> for AuthRoleToken {
    fn from(id: RoleId) -> Self {
        Self::new("role", id)
    }
}

impl From<PermissionSetId> for AuthRoleToken {
    fn from(id: PermissionSetId) -> Self {
        Self::new("permission_set", id)
    }
}

impl From<&RoleId> for AuthRoleToken {
    fn from(id: &RoleId) -> Self {
        (*id).into()
    }
}

impl From<&PermissionSetId> for AuthRoleToken {
    fn from(id: &PermissionSetId) -> Self {
        (*id).into()
    }
}

impl Display for AuthRoleToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.id)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Permission<O, A> {
    object: O,
    action: A,
}

impl<O, A> Permission<O, A> {
    pub const fn new(object: O, action: A) -> Self {
        Self { object, action }
    }

    pub fn object(&self) -> &O {
        &self.object
    }

    pub fn action(&self) -> &A {
        &self.action
    }
}

impl<O, A> From<&ActionMapping> for Permission<O, A>
where
    O: FromStr,
    A: FromStr,
{
    fn from(action: &ActionMapping) -> Self {
        Permission::new(
            action
                .all_objects_name()
                .parse()
                .map_err(|_| ())
                .expect("Could not parse object"),
            action
                .action_name()
                .parse()
                .map_err(|_| ())
                .expect("Could not parse action"),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccessAction {
    User(UserAction),
    Role(RoleAction),
    PermissionSet(PermissionSetAction),
}

impl CoreAccessAction {
    pub const ROLE_CREATE: Self = CoreAccessAction::Role(RoleAction::Create);
    pub const ROLE_UPDATE: Self = CoreAccessAction::Role(RoleAction::Update);
    pub const ROLE_LIST: Self = CoreAccessAction::Role(RoleAction::List);
    pub const ROLE_READ: Self = CoreAccessAction::Role(RoleAction::Read);

    pub const USER_CREATE: Self = CoreAccessAction::User(UserAction::Create);
    pub const USER_READ: Self = CoreAccessAction::User(UserAction::Read);
    pub const USER_LIST: Self = CoreAccessAction::User(UserAction::List);
    pub const USER_UPDATE_ROLE: Self = CoreAccessAction::User(UserAction::UpdateRole);

    pub const PERMISSION_SET_LIST: Self =
        CoreAccessAction::PermissionSet(PermissionSetAction::List);

    pub fn actions() -> Vec<ActionMapping> {
        use CoreAccessActionDiscriminants::*;
        use strum::VariantArray;

        CoreAccessActionDiscriminants::VARIANTS
            .iter()
            .flat_map(|&discriminant| match discriminant {
                User => map_action!(access, User, UserAction),
                Role => map_action!(access, Role, RoleAction),
                PermissionSet => {
                    map_action!(access, PermissionSet, PermissionSetAction)
                }
            })
            .collect()
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum RoleAction {
    Create,
    Update,
    Read,
    List,
}

impl ActionPermission for RoleAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_ACCESS_VIEWER,

            Self::Create | Self::Update => PERMISSION_SET_ACCESS_WRITER,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum PermissionSetAction {
    List,
}

impl ActionPermission for PermissionSetAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::List => PERMISSION_SET_ACCESS_VIEWER,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum UserAction {
    Read,
    Create,
    List,
    Update,
    UpdateRole,
}

impl ActionPermission for UserAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read | Self::List => PERMISSION_SET_ACCESS_VIEWER,

            Self::Create | Self::Update | Self::UpdateRole => PERMISSION_SET_ACCESS_WRITER,
        }
    }
}

impl Display for CoreAccessAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreAccessActionDiscriminants::from(self))?;
        use CoreAccessAction::*;
        match self {
            User(action) => action.fmt(f),
            Role(action) => action.fmt(f),
            PermissionSet(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreAccessAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreAccessActionDiscriminants::*;
        let res = match entity.parse()? {
            User => CoreAccessAction::from(action.parse::<UserAction>()?),
            Role => CoreAccessAction::from(action.parse::<RoleAction>()?),
            PermissionSet => CoreAccessAction::from(action.parse::<PermissionSetAction>()?),
        };
        Ok(res)
    }
}

impl From<UserAction> for CoreAccessAction {
    fn from(action: UserAction) -> Self {
        CoreAccessAction::User(action)
    }
}

impl From<RoleAction> for CoreAccessAction {
    fn from(action: RoleAction) -> Self {
        CoreAccessAction::Role(action)
    }
}

impl From<PermissionSetAction> for CoreAccessAction {
    fn from(action: PermissionSetAction) -> Self {
        CoreAccessAction::PermissionSet(action)
    }
}

pub type UserAllOrOne = AllOrOne<UserId>;
pub type RoleAllOrOne = AllOrOne<RoleId>;
pub type PermissionSetAllOrOne = AllOrOne<PermissionSetId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants, strum::EnumCount)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreAccessObject {
    User(UserAllOrOne),
    Role(RoleAllOrOne),
    PermissionSet(PermissionSetAllOrOne),
}

impl CoreAccessObject {
    pub const fn all_roles() -> CoreAccessObject {
        CoreAccessObject::Role(AllOrOne::All)
    }
    pub const fn role(id: RoleId) -> CoreAccessObject {
        CoreAccessObject::Role(AllOrOne::ById(id))
    }

    pub const fn all_permission_sets() -> CoreAccessObject {
        CoreAccessObject::PermissionSet(AllOrOne::All)
    }

    pub const fn all_users() -> CoreAccessObject {
        CoreAccessObject::User(AllOrOne::All)
    }
    pub fn user(id: impl Into<Option<UserId>>) -> CoreAccessObject {
        match id.into() {
            Some(id) => CoreAccessObject::User(AllOrOne::ById(id)),
            None => CoreAccessObject::all_users(),
        }
    }
}

impl Display for CoreAccessObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreAccessObjectDiscriminants::from(self);
        use CoreAccessObject::*;
        match self {
            User(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            Role(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
            PermissionSet(obj_ref) => write!(f, "{discriminant}/{obj_ref}"),
        }
    }
}

impl FromStr for CoreAccessObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreAccessObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            User => {
                let obj_ref = id.parse().map_err(|_| "could not parse UserObject")?;
                CoreAccessObject::User(obj_ref)
            }
            Role => {
                let obj_ref = id.parse().map_err(|_| "could not parse RoleObject")?;
                CoreAccessObject::Role(obj_ref)
            }
            PermissionSet => {
                let obj_ref = id
                    .parse()
                    .map_err(|_| "could not parse PermissionSetObject")?;
                CoreAccessObject::PermissionSet(obj_ref)
            }
        };
        Ok(res)
    }
}
