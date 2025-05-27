use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{action_description::*, AllOrOne};

#[cfg(feature = "governance")]
es_entity::entity_id! {
    UserId;
    UserId => governance::CommitteeMemberId,
}
#[cfg(not(feature = "governance"))]
es_entity::entity_id! { UserId }

es_entity::entity_id! { AuthenticationId, PermissionSetId, RoleId }

pub const ROLE_NAME_SUPERUSER: &str = "superuser";

pub const ACCESS_WRITER: &str = "access_writer";
pub const ACCESS_READER: &str = "access_reader";

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
pub struct Permission {
    object: String,
    action: String,
}

impl Permission {
    pub const fn new(object: String, action: String) -> Self {
        Self { object, action }
    }

    pub fn object(&self) -> &str {
        &self.object
    }

    pub fn action(&self) -> &str {
        &self.action
    }
}

impl From<ActionDescription<FullPath>> for Permission {
    fn from(action: ActionDescription<FullPath>) -> Self {
        Permission::new(action.all_objects_name(), action.action_name())
    }
}

impl From<&ActionDescription<FullPath>> for Permission {
    fn from(action: &ActionDescription<FullPath>) -> Self {
        Permission::new(action.all_objects_name(), action.action_name())
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
    pub const USER_ASSIGN_ROLE: Self = CoreAccessAction::User(UserAction::AssignRole);
    pub const USER_REVOKE_ROLE: Self = CoreAccessAction::User(UserAction::RevokeRole);
    pub const USER_UPDATE_AUTHENTICATION_ID: Self =
        CoreAccessAction::User(UserAction::UpdateAuthenticationId);

    pub const PERMISSION_SET_LIST: Self =
        CoreAccessAction::PermissionSet(PermissionSetAction::List);

    pub fn entities() -> Vec<(
        CoreAccessActionDiscriminants,
        Vec<ActionDescription<NoPath>>,
    )> {
        use CoreAccessActionDiscriminants::*;

        let mut result = vec![];

        for entity in <CoreAccessActionDiscriminants as strum::VariantArray>::VARIANTS {
            let actions = match entity {
                User => UserAction::describe(),
                Role => RoleAction::describe(),
                PermissionSet => PermissionSetAction::describe(),
            };

            result.push((*entity, actions));
        }

        result
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

impl RoleAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[ACCESS_WRITER]),
                Self::Update => ActionDescription::new(variant, &[ACCESS_WRITER]),
                Self::Read => ActionDescription::new(variant, &[ACCESS_READER, ACCESS_WRITER]),
                Self::List => ActionDescription::new(variant, &[ACCESS_READER, ACCESS_WRITER]),
            };
            res.push(action_description);
        }

        res
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum PermissionSetAction {
    List,
}

impl PermissionSetAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::List => ActionDescription::new(variant, &[ACCESS_READER, ACCESS_WRITER]),
            };
            res.push(action_description);
        }

        res
    }
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum UserAction {
    Read,
    Create,
    List,
    Update,
    AssignRole,
    RevokeRole,
    UpdateAuthenticationId,
}

impl UserAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[ACCESS_WRITER]),
                Self::Read => ActionDescription::new(variant, &[ACCESS_READER, ACCESS_WRITER]),
                Self::List => ActionDescription::new(variant, &[ACCESS_READER, ACCESS_WRITER]),
                Self::Update => ActionDescription::new(variant, &[ACCESS_WRITER]),
                Self::AssignRole => ActionDescription::new(variant, &[ACCESS_WRITER]),
                Self::RevokeRole => ActionDescription::new(variant, &[ACCESS_WRITER]),
                Self::UpdateAuthenticationId => ActionDescription::new(variant, &[ACCESS_WRITER]),
            };
            res.push(action_description);
        }

        res
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
            User(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Role(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            PermissionSet(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
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
