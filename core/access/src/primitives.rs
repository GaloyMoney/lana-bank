use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub use audit::AuditInfo;
pub use authz::{AllOrOne, action_description::*};

#[cfg(feature = "governance")]
es_entity::entity_id! {
    UserId;
    UserId => governance::CommitteeMemberId,
}
#[cfg(not(feature = "governance"))]
es_entity::entity_id! { UserId }

es_entity::entity_id! { AuthenticationId, PermissionSetId, RoleId }

pub const ROLE_NAME_SUPERUSER: &str = "superuser";

pub const PERMISSION_SET_ACCESS_WRITER: &str = "access_writer";
pub const PERMISSION_SET_ACCESS_VIEWER: &str = "access_viewer";

/// Expands permission set names to include hierarchical permissions.
/// When a "writer" permission is requested, the corresponding "viewer" permission
/// is automatically included, since writers should be able to read.
pub fn expand_permission_sets_with_hierarchy<'a>(
    permission_set_names: &'a [&'a str],
) -> Vec<&'a str> {
    let mut expanded = std::collections::HashSet::new();

    for &permission in permission_set_names {
        expanded.insert(permission);

        // Add implied permissions: writer permissions include viewer permissions
        match permission {
            PERMISSION_SET_ACCESS_WRITER => {
                expanded.insert(PERMISSION_SET_ACCESS_VIEWER);
            }
            "accounting_writer" => {
                expanded.insert("accounting_viewer");
            }
            "deposit_writer" => {
                expanded.insert("deposit_viewer");
            }
            "credit_writer" => {
                expanded.insert("credit_viewer");
            }
            "customer_writer" => {
                expanded.insert("customer_viewer");
            }
            "custody_writer" => {
                expanded.insert("custody_viewer");
            }
            "governance_writer" => {
                expanded.insert("governance_viewer");
            }
            "report_writer" => {
                expanded.insert("report_viewer");
            }
            // TODO: Add other module hierarchies as we migrate them:
            // etc.
            _ => {} // No hierarchy for viewer permissions or other types
        }
    }

    expanded.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_expansion_includes_viewer_for_writer() {
        // Test that when we request ACCESS_WRITER, we automatically get ACCESS_VIEWER too
        let input = &[PERMISSION_SET_ACCESS_WRITER];
        let result = expand_permission_sets_with_hierarchy(input);

        assert!(result.contains(&PERMISSION_SET_ACCESS_WRITER));
        assert!(result.contains(&PERMISSION_SET_ACCESS_VIEWER));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_hierarchy_expansion_viewer_only() {
        // Test that when we request ACCESS_VIEWER, we only get ACCESS_VIEWER
        let input = &[PERMISSION_SET_ACCESS_VIEWER];
        let result = expand_permission_sets_with_hierarchy(input);

        assert!(result.contains(&PERMISSION_SET_ACCESS_VIEWER));
        assert!(!result.contains(&PERMISSION_SET_ACCESS_WRITER));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_hierarchy_expansion_no_duplicates() {
        // Test that if we request both writer and viewer, we don't get duplicates
        let input = &[PERMISSION_SET_ACCESS_WRITER, PERMISSION_SET_ACCESS_VIEWER];
        let result = expand_permission_sets_with_hierarchy(input);

        assert!(result.contains(&PERMISSION_SET_ACCESS_WRITER));
        assert!(result.contains(&PERMISSION_SET_ACCESS_VIEWER));
        assert_eq!(result.len(), 2); // Should not have duplicates
    }

    #[test]
    fn test_new_action_description_api() {
        // Test the new simplified ActionDescription API
        let action_desc = ActionDescription::new2(RoleAction::Read, PERMISSION_SET_ACCESS_VIEWER);

        assert_eq!(action_desc.permission_sets().len(), 1);
        assert_eq!(
            action_desc.permission_sets()[0],
            PERMISSION_SET_ACCESS_VIEWER
        );
    }

    #[test]
    fn test_complete_migration_simplified_permissions() {
        // Test that all action types now use simplified single-permission approach
        let user_actions = UserAction::describe_v2();
        let role_actions = RoleAction::describe_v2();
        let permission_set_actions = PermissionSetAction::describe_v2();

        // All actions should now have exactly 1 permission set (no more arrays)
        for action in user_actions {
            assert_eq!(action.permission_sets().len(), 1);
        }
        for action in role_actions {
            assert_eq!(action.permission_sets().len(), 1);
        }
        for action in permission_set_actions {
            assert_eq!(action.permission_sets().len(), 1);
        }

        // Verify specific mappings: reads require viewer, writes require writer
        let read_actions = [
            UserAction::describe_v2()
                .into_iter()
                .find(|a| a.permission_sets()[0] == PERMISSION_SET_ACCESS_VIEWER),
            RoleAction::describe_v2()
                .into_iter()
                .find(|a| a.permission_sets()[0] == PERMISSION_SET_ACCESS_VIEWER),
        ];

        // Should have read actions that only require viewer permission
        assert!(read_actions.iter().any(|a| a.is_some()));
    }
}

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

impl<O, A> From<&ActionDescription<FullPath>> for Permission<O, A>
where
    O: FromStr,
    A: FromStr,
{
    fn from(action: &ActionDescription<FullPath>) -> Self {
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
                User => UserAction::describe_v2(), // Using new simplified API
                Role => RoleAction::describe_v2(), // Using new simplified API
                PermissionSet => PermissionSetAction::describe_v2(), // Using new simplified API
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
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_ACCESS_WRITER]),
                Self::Update => ActionDescription::new(variant, &[PERMISSION_SET_ACCESS_WRITER]),
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_ACCESS_VIEWER, PERMISSION_SET_ACCESS_WRITER],
                ),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_ACCESS_VIEWER, PERMISSION_SET_ACCESS_WRITER],
                ),
            };
            res.push(action_description);
        }

        res
    }

    /// New simplified approach: each action specifies the minimum required permission.
    /// Hierarchy is handled when assigning permissions to roles.
    pub fn describe_v2() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_WRITER),
                Self::Update => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_WRITER),
                Self::Read => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_VIEWER),
                Self::List => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_VIEWER),
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
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_ACCESS_VIEWER, PERMISSION_SET_ACCESS_WRITER],
                ),
            };
            res.push(action_description);
        }

        res
    }

    /// New simplified approach: each action specifies the minimum required permission.
    /// Hierarchy is handled when assigning permissions to roles.
    pub fn describe_v2() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::List => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_VIEWER),
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
    UpdateRole,
    UpdateAuthenticationId,
}

impl UserAction {
    pub fn describe() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new(variant, &[PERMISSION_SET_ACCESS_WRITER]),
                Self::Read => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_ACCESS_VIEWER, PERMISSION_SET_ACCESS_WRITER],
                ),
                Self::List => ActionDescription::new(
                    variant,
                    &[PERMISSION_SET_ACCESS_VIEWER, PERMISSION_SET_ACCESS_WRITER],
                ),
                Self::Update => ActionDescription::new(variant, &[PERMISSION_SET_ACCESS_WRITER]),
                Self::UpdateRole => {
                    ActionDescription::new(variant, &[PERMISSION_SET_ACCESS_WRITER])
                }
                Self::UpdateAuthenticationId => {
                    ActionDescription::new(variant, &[PERMISSION_SET_ACCESS_WRITER])
                }
            };
            res.push(action_description);
        }

        res
    }

    /// New simplified approach: each action specifies the minimum required permission.
    /// Hierarchy is handled when assigning permissions to roles.
    pub fn describe_v2() -> Vec<ActionDescription<NoPath>> {
        let mut res = vec![];

        for variant in <Self as strum::VariantArray>::VARIANTS {
            let action_description = match variant {
                Self::Create => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_WRITER),
                Self::Read => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_VIEWER),
                Self::List => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_VIEWER),
                Self::Update => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_WRITER),
                Self::UpdateRole => ActionDescription::new2(variant, PERMISSION_SET_ACCESS_WRITER),
                Self::UpdateAuthenticationId => {
                    ActionDescription::new2(variant, PERMISSION_SET_ACCESS_WRITER)
                }
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
