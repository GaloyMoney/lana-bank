use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{Role, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "UserId")]
pub enum UserEvent {
    Initialized {
        id: UserId,
        email: String,
        audit_info: AuditInfo,
    },
    AuthenticationIdUpdated {
        authentication_id: AuthenticationId,
    },
    RoleGranted {
        id: RoleId,
        audit_info: AuditInfo,
    },
    RoleRevoked {
        id: RoleId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct User {
    pub id: UserId,
    pub email: String,
    #[builder(setter(strip_option), default)]
    pub authentication_id: Option<AuthenticationId>,
    events: EntityEvents<UserEvent>,
}

impl User {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    /// Sets user's role to `role`. Returns previous role or `None`
    /// if no role was previously set.
    pub(crate) fn update_role(
        &mut self,
        role: &Role,
        audit_info: AuditInfo,
    ) -> Idempotent<Option<RoleId>> {
        match self.current_role() {
            Some(current) if role.id == current => Idempotent::Ignored,
            previous => {
                if let Some(previous) = previous {
                    self.events.push(UserEvent::RoleRevoked {
                        id: previous,
                        audit_info: audit_info.clone(),
                    });
                }

                self.events.push(UserEvent::RoleGranted {
                    id: role.id,
                    audit_info,
                });

                Idempotent::Executed(previous)
            }
        }
    }

    /// Revokes role this user currently has. Returns previous role.
    pub(crate) fn revoke_role(&mut self, audit_info: AuditInfo) -> Idempotent<RoleId> {
        match self.current_role() {
            None => Idempotent::Ignored,
            Some(previous) => {
                self.events.push(UserEvent::RoleRevoked {
                    id: previous,
                    audit_info,
                });

                Idempotent::Executed(previous)
            }
        }
    }

    /// Returns the role currently assigned to this user. Returns `None`
    /// if no role has been assigned to the user or previous role has been revoked.
    pub fn current_role(&self) -> Option<RoleId> {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                UserEvent::RoleGranted { id: role_id, .. } => Some(Some(*role_id)),
                UserEvent::RoleRevoked { .. } => Some(None),
                _ => None,
            })
            .flatten()
    }

    pub fn update_authentication_id(
        &mut self,
        authentication_id: AuthenticationId,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            UserEvent::AuthenticationIdUpdated { authentication_id: existing_id } if existing_id == &authentication_id
        );
        self.authentication_id = Some(authentication_id);
        self.events
            .push(UserEvent::AuthenticationIdUpdated { authentication_id });
        Idempotent::Executed(())
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
                UserEvent::RoleGranted { .. } => (),
                UserEvent::RoleRevoked { .. } => (),
                UserEvent::AuthenticationIdUpdated { authentication_id } => {
                    builder = builder.authentication_id(*authentication_id);
                }
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
    pub(super) audit_info: AuditInfo,
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
                audit_info: self.audit_info,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use audit::{AuditEntryId, AuditInfo};
    use es_entity::{Idempotent, IntoEvents as _, TryFromEvents as _};

    use crate::{NewRole, Role, RoleId, UserId};

    use super::{NewUser, User};

    fn audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn new_user() -> User {
        let new_user = NewUser::builder()
            .id(UserId::new())
            .email("email")
            .audit_info(audit_info())
            .build()
            .unwrap();

        User::try_from_events(new_user.into_events()).unwrap()
    }

    fn new_role() -> Role {
        Role::try_from_events(
            NewRole::builder()
                .id(RoleId::new())
                .name("a role".to_string())
                .audit_info(audit_info())
                .build()
                .unwrap()
                .into_events(),
        )
        .unwrap()
    }

    #[test]
    fn user_updating_role() {
        let mut user = new_user();
        assert_eq!(user.current_role(), None);

        assert!(user.revoke_role(audit_info()).was_ignored());

        let role_1 = new_role();
        let previous = user.update_role(&role_1, audit_info());
        assert!(matches!(previous, Idempotent::Executed(None)));

        let previous = user.update_role(&role_1, audit_info());
        assert!(matches!(previous, Idempotent::Ignored));

        let role_2 = new_role();
        let previous = user.update_role(&role_2, audit_info());
        assert!(matches!(previous, Idempotent::Executed(Some(id)) if id == role_1.id));
        assert_eq!(user.current_role(), Some(role_2.id));

        let previous = user.revoke_role(audit_info());
        assert!(matches!(previous, Idempotent::Executed(id) if id == role_2.id));
        assert_eq!(user.current_role(), None);
    }
}
