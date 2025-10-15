use chrono::{DateTime, NaiveDate, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LedgerClosingId")]
pub enum LedgerClosingEvent {
    MetadataConfigured {
        id: LedgerClosingId,
        root_account_set_id: CalaAccountSetId,
        opened_as_of: NaiveDate,
    },
    AccountingPeriodClosed {
        closed_as_of: NaiveDate,
        closing_tx_id: Option<CalaTxId>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LedgerClosing {
    pub id: LedgerClosingId,
    pub root_account_set_id: CalaAccountSetId,
    pub opened_as_of: NaiveDate,
    pub last_closing: PeriodClosing,
    events: EntityEvents<LedgerClosingEvent>,
}

impl LedgerClosing {}

impl TryFromEvents<LedgerClosingEvent> for LedgerClosing {
    fn try_from_events(events: EntityEvents<LedgerClosingEvent>) -> Result<Self, EsEntityError> {
        let mut builder = LedgerClosingBuilder::default();

        for event in events.iter_all() {
            match event {
                LedgerClosingEvent::MetadataConfigured {
                    id,
                    root_account_set_id,
                    opened_as_of,
                } => {
                    let last_monthly_closed_as_of = opened_as_of
                        .pred_opt()
                        .expect("Failed to get day prior to opening date");
                    let monthly_closing =
                        PeriodClosing::new(last_monthly_closed_as_of, None);
                    builder = builder
                        .id(*id)
                        .root_account_set_id(*root_account_set_id)
                        .opened_as_of(*opened_as_of)
                        .last_closing(monthly_closing);
                }
                LedgerClosingEvent::AccountingPeriodClosed {
                    closed_as_of, closing_tx_id } => {
                    builder = builder.last_closing(PeriodClosing::new(*closed_as_of, *closing_tx_id));
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewLedgerClosing {
    #[builder(setter(into))]
    pub(super) id: LedgerClosingId,
    #[builder(setter(into))]
    pub(super) root_account_set_id: CalaAccountSetId,
    pub(super) opened_as_of: NaiveDate,
}

impl NewLedgerClosing {
    pub fn builder() -> NewLedgerClosingBuilder {
        NewLedgerClosingBuilder::default()
    }
}

impl IntoEvents<LedgerClosingEvent> for NewLedgerClosing {
    fn into_events(self) -> EntityEvents<LedgerClosingEvent> {
        EntityEvents::init(
            self.id,
            [LedgerClosingEvent::MetadataConfigured {
                id: self.id,
                root_account_set_id: self.root_account_set_id,
                opened_as_of: self.opened_as_of,
            }],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodClosing {
    pub closed_as_of: chrono::NaiveDate,
    pub closing_tx_id: Option<CalaTxId>,
}

impl PeriodClosing {
    fn new(effective: NaiveDate, closing_tx_id: Option<CalaTxId>) -> Self {
        Self {
            closed_as_of: effective,
            closing_tx_id,
        }
    }
}
