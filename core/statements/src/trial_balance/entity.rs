use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;

use es_entity::*;

use crate::primitives::{LedgerAccountSetId, TrialBalanceStatementId};

pub use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "TrialBalanceStatementId")]
pub enum TrialBalanceStatementEvent {
    Initialized {
        id: TrialBalanceStatementId,
        name: String,
        reference: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct TrialBalanceStatement {
    pub id: TrialBalanceStatementId,
    pub name: String,
    pub reference: String,
    pub account_set_id: LedgerAccountSetId,
    pub(super) events: EntityEvents<TrialBalanceStatementEvent>,
}

impl TryFromEvents<TrialBalanceStatementEvent> for TrialBalanceStatement {
    fn try_from_events(
        events: EntityEvents<TrialBalanceStatementEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = TrialBalanceStatementBuilder::default();
        for event in events.iter_all() {
            match event {
                TrialBalanceStatementEvent::Initialized {
                    id,
                    name,
                    reference,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .name(name.to_string())
                        .reference(reference.to_string())
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewTrialBalanceStatement {
    #[builder(setter(into))]
    pub(super) id: TrialBalanceStatementId,
    pub(super) name: String,
    pub(super) reference: String,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewTrialBalanceStatement {
    pub fn builder() -> NewTrialBalanceStatementBuilder {
        NewTrialBalanceStatementBuilder::default()
    }
}

impl IntoEvents<TrialBalanceStatementEvent> for NewTrialBalanceStatement {
    fn into_events(self) -> EntityEvents<TrialBalanceStatementEvent> {
        EntityEvents::init(
            self.id,
            [TrialBalanceStatementEvent::Initialized {
                id: self.id,
                name: self.name,
                reference: self.reference,
                audit_info: self.audit_info,
            }],
        )
    }
}
