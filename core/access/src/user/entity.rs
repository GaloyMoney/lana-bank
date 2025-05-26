use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{primitives::*, Role};

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
    RoleAssigned {
        id: RoleId,
        name: String,
        audit_info: AuditInfo,
    },
    RoleRevoked {
        id: RoleId,
        name: String,
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

    pub(crate) fn assign_role(&mut self, role: &Role, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            UserEvent::RoleAssigned { id: assigned, .. } if *assigned == role.id,
            => UserEvent::RoleRevoked { id: revoked,.. } if *revoked == role.id
        );

        self.events.push(UserEvent::RoleAssigned {
            id: role.id,
            name: role.name.clone(),
            audit_info,
        });
        Idempotent::Executed(())
    }

    pub(crate) fn revoke_role(&mut self, role: &Role, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            UserEvent::RoleRevoked { id: revoked, .. } if *revoked == role.id,
            => UserEvent::RoleAssigned { id: assigned,.. } if *assigned == role.id
        );

        self.events.push(UserEvent::RoleRevoked {
            id: role.id,
            name: role.name.clone(),
            audit_info,
        });

        Idempotent::Executed(())
    }

    pub fn current_role(&self) -> Option<RoleId> {
        let mut res = None;
        for event in self.events.iter_all() {
            match event {
                UserEvent::RoleAssigned { id: role_id, .. } => {
                    let _ = res.insert(*role_id);
                }
                UserEvent::RoleRevoked { .. } => {
                    res.take();
                }
                _ => {}
            }
        }
        res
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
                UserEvent::RoleAssigned { .. } => (),
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
