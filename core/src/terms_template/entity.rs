use crate::{entity::*, loan::TermValues, primitives::*};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TermsTemplateEvent {
    Initialized {
        id: LoanTermsId,
        name: String,
        values: TermValues,
        audit_info: AuditInfo,
    },
    // TODO add update event.
}

impl TermsTemplateEvent {
    fn audit_info(&self) -> AuditInfo {
        match self {
            TermsTemplateEvent::Initialized { audit_info, .. } => *audit_info,
        }
    }
}

impl EntityEvent for TermsTemplateEvent {
    type EntityId = LoanTermsId;
    fn event_table_name() -> &'static str {
        "terms_template_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct TermsTemplate {
    pub id: LoanTermsId,
    pub name: String,
    pub values: TermValues,
    pub(super) events: EntityEvents<TermsTemplateEvent>,
}

impl Entity for TermsTemplate {
    type Event = TermsTemplateEvent;
}

impl TermsTemplate {
    pub fn audit_info(&self) -> Vec<AuditInfo> {
        self.events.iter().map(|e| e.audit_info()).collect()
    }
}

impl TryFrom<EntityEvents<TermsTemplateEvent>> for TermsTemplate {
    type Error = EntityError;

    fn try_from(events: EntityEvents<TermsTemplateEvent>) -> Result<Self, Self::Error> {
        let mut builder = TermsTemplateBuilder::default();

        for event in events.iter() {
            match event {
                TermsTemplateEvent::Initialized {
                    id, name, values, ..
                } => {
                    builder = builder.id(*id).name(name.clone()).values(*values);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Builder)]
pub struct NewTermsTemplate {
    #[builder(setter(into))]
    pub id: LoanTermsId,
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub values: TermValues,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewTermsTemplate {
    pub fn builder() -> NewTermsTemplateBuilder {
        NewTermsTemplateBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<TermsTemplateEvent> {
        EntityEvents::init(
            self.id,
            [TermsTemplateEvent::Initialized {
                id: self.id,
                name: self.name,
                values: self.values,
                audit_info: self.audit_info,
            }],
        )
    }
}
