use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::AllOrOne;

#[cfg(feature = "governance")]
es_entity::entity_id! {
    UserId;
    UserId => governance::CommitteeMemberId,
}
#[cfg(not(feature = "governance"))]
es_entity::entity_id! { UserId }

es_entity::entity_id! { AuthenticationId, AccessGroupId, RoleId }

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleName {
    Superuser,
    Admin,
    Accountant,
    BankManager,
    #[serde(untagged)]
    Other(String),
}
impl RoleName {
    pub fn new(role_name: impl Into<String>) -> Self {
        RoleName::Other(role_name.into())
    }

    pub fn name(&self) -> &str {
        match self {
            RoleName::Superuser => "superuser",
            RoleName::Admin => "admin",
            RoleName::Accountant => "accountant",
            RoleName::BankManager => "bank_manager",
            RoleName::Other(name) => name,
        }
    }
}

impl Display for RoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreUserAction {
    User(UserAction),
    Role(RoleAction),
}

impl CoreUserAction {
    pub const ROLE_CREATE: Self = CoreUserAction::Role(RoleAction::Create);
    pub const ROLE_UPDATE: Self = CoreUserAction::Role(RoleAction::Update);

    pub const USER_CREATE: Self = CoreUserAction::User(UserAction::Create);
    pub const USER_READ: Self = CoreUserAction::User(UserAction::Read);
    pub const USER_LIST: Self = CoreUserAction::User(UserAction::List);
    pub const USER_ASSIGN_ROLE: Self = CoreUserAction::User(UserAction::AssignRole);
    pub const USER_REVOKE_ROLE: Self = CoreUserAction::User(UserAction::RevokeRole);
    pub const USER_UPDATE_AUTHENTICATION_ID: Self =
        CoreUserAction::User(UserAction::UpdateAuthenticationId);
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum RoleAction {
    Create,
    Update,
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
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

impl Display for CoreUserAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", CoreUserActionDiscriminants::from(self))?;
        use CoreUserAction::*;
        match self {
            User(action) => action.fmt(f),
            Role(action) => action.fmt(f),
        }
    }
}

impl FromStr for CoreUserAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreUserActionDiscriminants::*;
        let res = match entity.parse()? {
            User => CoreUserAction::from(action.parse::<UserAction>()?),
            Role => CoreUserAction::from(action.parse::<RoleAction>()?),
        };
        Ok(res)
    }
}

impl From<UserAction> for CoreUserAction {
    fn from(action: UserAction) -> Self {
        CoreUserAction::User(action)
    }
}

impl From<RoleAction> for CoreUserAction {
    fn from(action: RoleAction) -> Self {
        CoreUserAction::Role(action)
    }
}

pub type UserAllOrOne = AllOrOne<UserId>;
pub type RoleAllOrOne = AllOrOne<RoleId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreUserObject {
    User(UserAllOrOne),
    Role(RoleAllOrOne),
}

impl CoreUserObject {
    pub const fn all_roles() -> CoreUserObject {
        CoreUserObject::Role(AllOrOne::All)
    }
    pub const fn role(id: RoleId) -> CoreUserObject {
        CoreUserObject::Role(AllOrOne::ById(id))
    }

    pub const fn all_users() -> CoreUserObject {
        CoreUserObject::User(AllOrOne::All)
    }
    pub fn user(id: impl Into<Option<UserId>>) -> CoreUserObject {
        match id.into() {
            Some(id) => CoreUserObject::User(AllOrOne::ById(id)),
            None => CoreUserObject::all_users(),
        }
    }
}

impl Display for CoreUserObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = CoreUserObjectDiscriminants::from(self);
        use CoreUserObject::*;
        match self {
            User(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
            Role(obj_ref) => write!(f, "{}/{}", discriminant, obj_ref),
        }
    }
}

impl FromStr for CoreUserObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, id) = s.split_once('/').expect("missing slash");
        use CoreUserObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            User => {
                let obj_ref = id.parse().map_err(|_| "could not parse UserObject")?;
                CoreUserObject::User(obj_ref)
            }
            Role => {
                let obj_ref = id.parse().map_err(|_| "could not parse RoleObject")?;
                CoreUserObject::Role(obj_ref)
            }
        };
        Ok(res)
    }
}
