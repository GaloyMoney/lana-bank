use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::{CalaAccountId, ObligationId, UsdCents};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ObligationId")]
pub enum ObligationEvent {
    Initialized {
        id: ObligationId,
        amount: UsdCents,
        account_to_be_debited_id: CalaAccountId,
        account_to_be_credited_id: CalaAccountId,
        due_date: DateTime<Utc>,
        overdue_date: Option<DateTime<Utc>>,
        default_date: Option<DateTime<Utc>>,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Obligation {
    pub id: ObligationId,
    pub(super) events: EntityEvents<ObligationEvent>,
}

impl Obligation {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }
}

impl TryFromEvents<ObligationEvent> for Obligation {
    fn try_from_events(events: EntityEvents<ObligationEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ObligationBuilder::default();
        for event in events.iter_all() {
            match event {
                ObligationEvent::Initialized { id, .. } => {
                    builder = builder.id(*id);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewObligation {
    #[builder(setter(into))]
    pub(super) id: ObligationId,
    #[builder(setter(into))]
    pub(super) amount: UsdCents,
    #[builder(setter(into))]
    account_to_be_debited_id: CalaAccountId,
    #[builder(setter(into))]
    account_to_be_credited_id: CalaAccountId,
    due_date: DateTime<Utc>,
    overdue_date: Option<DateTime<Utc>>,
    default_date: Option<DateTime<Utc>>,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewObligation {
    pub fn builder() -> NewObligationBuilder {
        NewObligationBuilder::default()
    }
}

impl IntoEvents<ObligationEvent> for NewObligation {
    fn into_events(self) -> EntityEvents<ObligationEvent> {
        EntityEvents::init(
            self.id,
            [ObligationEvent::Initialized {
                id: self.id,
                amount: self.amount,
                account_to_be_debited_id: self.account_to_be_debited_id,
                account_to_be_credited_id: self.account_to_be_credited_id,
                due_date: self.due_date,
                overdue_date: self.overdue_date,
                default_date: self.default_date,
                audit_info: self.audit_info,
            }],
        )
    }
}
