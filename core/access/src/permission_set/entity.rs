use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use es_entity::*;

use crate::primitives::{Permission, PermissionSetId};

#[derive(Eq, PartialEq, Hash, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PermissionValues {
    object: String,
    action: String,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "PermissionSetId")]
pub enum PermissionSetEvent {
    Initialized {
        id: PermissionSetId,
        name: String,
        permissions: HashSet<PermissionValues>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct PermissionSet {
    pub id: PermissionSetId,
    pub name: String,
    events: EntityEvents<PermissionSetEvent>,
}

impl PermissionSet {
    /// Returns all permissions assigned to this Permission Set.
    pub fn permissions<O, A>(&self) -> impl Iterator<Item = Permission<O, A>> + '_
    where
        O: std::str::FromStr,
        A: std::str::FromStr,
    {
        self.events.iter_all().flat_map(|event| match event {
            PermissionSetEvent::Initialized { permissions, .. } => {
                permissions.iter().map(|permission| {
                    Permission::new(
                        permission
                            .object
                            .parse()
                            .map_err(|_| ())
                            .expect("Could not parse object"),
                        permission
                            .action
                            .parse()
                            .map_err(|_| ())
                            .expect("Could not parse action"),
                    )
                })
            }
        })
    }
}

impl TryFromEvents<PermissionSetEvent> for PermissionSet {
    fn try_from_events(events: EntityEvents<PermissionSetEvent>) -> Result<Self, EsEntityError> {
        let mut builder = PermissionSetBuilder::default();

        for event in events.iter_all() {
            match event {
                PermissionSetEvent::Initialized { id, name, .. } => {
                    builder = builder.id(*id).name(name.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct NewPermissionSet {
    #[builder(setter(into))]
    pub(super) id: PermissionSetId,
    #[builder(setter(into))]
    pub(super) name: String,
    #[builder(setter(custom))]
    permissions: HashSet<PermissionValues>,
}

impl NewPermissionSet {
    pub fn builder() -> NewPermissionSetBuilder {
        Default::default()
    }
}

impl NewPermissionSetBuilder {
    pub fn permissions<O: std::fmt::Display, A: std::fmt::Display>(
        mut self,
        permissions: impl IntoIterator<Item = Permission<O, A>>,
    ) -> Self {
        let permissions: HashSet<PermissionValues> = permissions
            .into_iter()
            .map(|permission| PermissionValues {
                object: permission.object().to_string(),
                action: permission.action().to_string(),
            })
            .collect();
        self.permissions = Some(permissions);
        self
    }
}

impl IntoEvents<PermissionSetEvent> for NewPermissionSet {
    fn into_events(self) -> EntityEvents<PermissionSetEvent> {
        EntityEvents::init(
            self.id,
            [PermissionSetEvent::Initialized {
                id: self.id,
                name: self.name,
                permissions: self.permissions,
            }],
        )
    }
}
