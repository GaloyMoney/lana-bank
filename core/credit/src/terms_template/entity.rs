use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{TermValues, primitives::*};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "TermsTemplateId")]
pub enum TermsTemplateEvent {
    Initialized {
        id: TermsTemplateId,
        name: String,
        values: TermValues,
    },
    TermValuesUpdated {
        values: TermValues,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct TermsTemplate {
    pub id: TermsTemplateId,
    pub name: String,
    pub values: TermValues,
    events: EntityEvents<TermsTemplateEvent>,
}

impl TermsTemplate {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("TermsTemplate has never been persisted")
    }

    pub fn update_values(&mut self, new_values: TermValues) {
        self.events
            .push(TermsTemplateEvent::TermValuesUpdated { values: new_values });
        self.values = new_values;
    }
}

impl TryFromEvents<TermsTemplateEvent> for TermsTemplate {
    fn try_from_events(events: EntityEvents<TermsTemplateEvent>) -> Result<Self, EsEntityError> {
        let mut builder = TermsTemplateBuilder::default();

        for event in events.iter_all() {
            match event {
                TermsTemplateEvent::Initialized {
                    id, name, values, ..
                } => {
                    builder = builder.id(*id).name(name.clone()).values(*values);
                }
                TermsTemplateEvent::TermValuesUpdated { values, .. } => {
                    builder = builder.values(*values);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Builder)]
pub struct NewTermsTemplate {
    #[builder(setter(into))]
    pub id: TermsTemplateId,
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub values: TermValues,
}

impl NewTermsTemplate {
    pub fn builder() -> NewTermsTemplateBuilder {
        NewTermsTemplateBuilder::default()
    }
}

impl IntoEvents<TermsTemplateEvent> for NewTermsTemplate {
    fn into_events(self) -> EntityEvents<TermsTemplateEvent> {
        EntityEvents::init(
            self.id,
            [TermsTemplateEvent::Initialized {
                id: self.id,
                name: self.name,
                values: self.values,
            }],
        )
    }
}
