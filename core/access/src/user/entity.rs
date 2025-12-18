use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{Role, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "UserId")]
pub enum UserEvent {
    Initialized {
        id: UserId,
        email: String,
        role_id: RoleId,
    },
    RoleUpdated {
        role_id: RoleId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct User {
    pub id: UserId,
    pub email: String,
    events: EntityEvents<UserEvent>,
}

impl User {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    /// Sets user's role to `role`. Returns previous role.
    pub(crate) fn update_role(&mut self, role: &Role) -> Idempotent<RoleId> {
        let current = self.current_role();
        if role.id == current {
            Idempotent::AlreadyApplied
        } else {
            self.events
                .push(UserEvent::RoleUpdated { role_id: role.id });

            Idempotent::Executed(current)
        }
    }

    /// Returns the role currently assigned to this user.
    /// Always returns a role since roles are mandatory from creation.
    pub fn current_role(&self) -> RoleId {
        self.events
            .iter_all()
            .rev()
            .map(|event| match event {
                UserEvent::RoleUpdated { role_id, .. } => *role_id,
                UserEvent::Initialized { role_id, .. } => *role_id,
            })
            .next()
            .expect("User must have a role assigned")
    }
}

impl core::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User: {}, email: {}", self.id, self.email)
    }
}

impl TryFromEvents<UserEvent> for User {
    fn try_from_events(events: EntityEvents<UserEvent>) -> Result<Self, EsEntityError> {
        let mut builder = UserBuilder::default();

        for event in events.iter_all() {
            match event {
                UserEvent::Initialized { id, email, .. } => {
                    builder = builder.id(*id).email(email.clone())
                }
                UserEvent::RoleUpdated { .. } => (),
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewUser {
    #[builder(setter(into))]
    pub(super) id: UserId,
    #[builder(setter(into))]
    pub(super) email: String,
    pub(super) role_id: RoleId,
}

impl NewUser {
    pub fn builder() -> NewUserBuilder {
        let user_id = UserId::new();

        let mut builder = NewUserBuilder::default();
        builder.id(user_id);
        builder
    }
}

impl IntoEvents<UserEvent> for NewUser {
    fn into_events(self) -> EntityEvents<UserEvent> {
        EntityEvents::init(
            self.id,
            [UserEvent::Initialized {
                id: self.id,
                email: self.email,
                role_id: self.role_id,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use es_entity::{Idempotent, IntoEvents as _, TryFromEvents as _};

    use crate::{NewRole, Role, RoleId, UserId};

    use super::{NewUser, User};

    fn new_user() -> User {
        let role = new_role();
        let new_user = NewUser::builder()
            .id(UserId::new())
            .email("email")
            .role_id(role.id)
            .build()
            .unwrap();

        User::try_from_events(new_user.into_events()).unwrap()
    }

    fn new_role() -> Role {
        Role::try_from_events(
            NewRole::builder()
                .id(RoleId::new())
                .name("a role".to_string())
                .build()
                .unwrap()
                .into_events(),
        )
        .unwrap()
    }

    #[test]
    fn user_updating_role() {
        let mut user = new_user();
        let initial_role = user.current_role();

        let role_1 = new_role();
        let role_2 = new_role();

        // Updating to the same role should be ignored
        let same_role_update = user.update_role(
            &Role::try_from_events(
                NewRole::builder()
                    .id(initial_role)
                    .name("initial role".to_string())
                    .build()
                    .unwrap()
                    .into_events(),
            )
            .unwrap(),
        );
        assert!(matches!(same_role_update, Idempotent::AlreadyApplied));
        assert_eq!(user.current_role(), initial_role);

        // Updating to a different role should return the previous role
        let role_change = user.update_role(&role_1);
        assert!(matches!(role_change, Idempotent::Executed(id) if id == initial_role));
        assert_eq!(user.current_role(), role_1.id);

        // Updating to another different role should return the previous role
        let second_role_change = user.update_role(&role_2);
        assert!(matches!(second_role_change, Idempotent::Executed(id) if id == role_1.id));
        assert_eq!(user.current_role(), role_2.id);
    }
}
