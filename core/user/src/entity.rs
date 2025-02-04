use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use std::collections::HashSet;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "UserId")]
pub enum UserEvent {
    Initialized {
        id: UserId,
        email: String,
        audit_info: AuditInfo,
    },
    AuthenticationIdSet {
        authentication_id: AuthenticationId,
    },
    RoleAssigned {
        role: Role,
        audit_info: AuditInfo,
    },
    RoleRevoked {
        role: Role,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub authentication_id: Option<AuthenticationId>,
    events: EntityEvents<UserEvent>,
}

impl User {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub(crate) fn assign_role(&mut self, role: Role, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            UserEvent::RoleAssigned { role: assigned, .. } if assigned == &role,
            => UserEvent::RoleRevoked { role: revoked,.. } if revoked == &role
        );

        self.events
            .push(UserEvent::RoleAssigned { role, audit_info });
        Idempotent::Executed(())
    }

    pub(crate) fn revoke_role(&mut self, role: Role, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            UserEvent::RoleRevoked { role: revoked, .. } if revoked == &role,
            => UserEvent::RoleAssigned { role: assigned,.. } if assigned == &role
        );

        self.events
            .push(UserEvent::RoleRevoked { role, audit_info });

        Idempotent::Executed(())
    }

    pub fn current_roles(&self) -> HashSet<Role> {
        let mut res = HashSet::new();
        for event in self.events.iter_all() {
            match event {
                UserEvent::RoleAssigned { role, .. } => {
                    res.insert(role.clone());
                }
                UserEvent::RoleRevoked { role, .. } => {
                    res.remove(role);
                }
                _ => {}
            }
        }
        res
    }

    pub fn set_authentication_id(&mut self, authentication_id: AuthenticationId) {
        self.events
            .push(UserEvent::AuthenticationIdSet { authentication_id });
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
                UserEvent::RoleAssigned { .. } => (),
                UserEvent::RoleRevoked { .. } => (),
                UserEvent::AuthenticationIdSet { .. } => (),
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
    #[builder(setter(into), default)]
    pub(super) authentication_id: Option<AuthenticationId>,
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
