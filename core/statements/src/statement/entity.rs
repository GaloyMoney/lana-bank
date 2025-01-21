use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::primitives::{LedgerAccountSetId, StatementId};

pub use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "StatementId")]
pub enum StatementEvent {
    Initialized {
        id: StatementId,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Statement {
    pub id: StatementId,
    pub name: String,
    pub reference: String,
    pub account_set_id: LedgerAccountSetId,
    pub(super) events: EntityEvents<StatementEvent>,
}

impl TryFromEvents<StatementEvent> for Statement {
    fn try_from_events(events: EntityEvents<StatementEvent>) -> Result<Self, EsEntityError> {
        let mut builder = StatementBuilder::default();
        for event in events.iter_all() {
            match event {
                StatementEvent::Initialized { id, reference, .. } => {
                    builder = builder.id(*id).reference(reference.to_string())
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewStatement {
    #[builder(setter(into))]
    pub(super) id: StatementId,
    pub(super) name: String,
    pub(super) reference: String,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewStatement {
    pub fn builder() -> NewStatementBuilder {
        NewStatementBuilder::default()
    }
}

impl IntoEvents<StatementEvent> for NewStatement {
    fn into_events(self) -> EntityEvents<StatementEvent> {
        EntityEvents::init(
            self.id,
            [StatementEvent::Initialized {
                id: self.id,
                name: self.name,
                reference: self.reference,
                audit_info: self.audit_info,
            }],
        )
    }
}
