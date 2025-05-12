use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::AllOrOne;

#[cfg(feature = "governance")]
es_entity::entity_id! {
    UserId;
    UserId => governance::CommitteeMemberId,
}
#[cfg(not(feature = "governance"))]
es_entity::entity_id! { UserId }

es_entity::entity_id! { AuthenticationId, RoleId }

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct RoleName(Cow<'static, str>);
impl RoleName {
    pub const SUPERUSER: RoleName = RoleName::new("superuser");

    pub const fn new(role_name: &'static str) -> Self {
        RoleName(Cow::Borrowed(role_name))
    }
}

impl Display for RoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreUserAction {
    User(UserEntityAction),
}

impl CoreUserAction {
    pub const USER_CREATE: Self = CoreUserAction::User(UserEntityAction::Create);
    pub const USER_READ: Self = CoreUserAction::User(UserEntityAction::Read);
    pub const USER_LIST: Self = CoreUserAction::User(UserEntityAction::List);
    pub const USER_ASSIGN_ROLE: Self = CoreUserAction::User(UserEntityAction::AssignRole);
    pub const USER_REVOKE_ROLE: Self = CoreUserAction::User(UserEntityAction::RevokeRole);
    pub const USER_UPDATE_AUTHENTICATION_ID: Self =
        CoreUserAction::User(UserEntityAction::UpdateAuthenticationId);
}

#[derive(PartialEq, Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum UserEntityAction {
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
        }
    }
}

impl FromStr for CoreUserAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, action) = s.split_once(':').expect("missing colon");
        use CoreUserActionDiscriminants::*;
        let res = match entity.parse()? {
            User => CoreUserAction::from(action.parse::<UserEntityAction>()?),
        };
        Ok(res)
    }
}

impl From<UserEntityAction> for CoreUserAction {
    fn from(action: UserEntityAction) -> Self {
        CoreUserAction::User(action)
    }
}

pub type UserAllOrOne = AllOrOne<UserId>;

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CoreUserObject {
    User(UserAllOrOne),
}

impl CoreUserObject {
    pub fn all_users() -> CoreUserObject {
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
        };
        Ok(res)
    }
}
