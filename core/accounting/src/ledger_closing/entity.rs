use chrono::NaiveDate;
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{CalaTxId, ChartId, LedgerClosingId};

use super::error::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LedgerClosingId")]
pub enum LedgerClosingEvent {
    Initialized {
        id: LedgerClosingId,
        accountant: Option<String>,
        closed_as_of: NaiveDate,
        closing_tx_id: Option<CalaTxId>,
        chart_id: ChartId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LedgerClosing {
    pub id: LedgerClosingId,
    pub chart_id: ChartId,
    pub last_closing: PeriodClosing,
    events: EntityEvents<LedgerClosingEvent>,
}

impl LedgerClosing {}

impl TryFromEvents<LedgerClosingEvent> for LedgerClosing {
    fn try_from_events(events: EntityEvents<LedgerClosingEvent>) -> Result<Self, EsEntityError> {
        let mut builder = LedgerClosingBuilder::default();

        for event in events.iter_all() {
            match event {
                LedgerClosingEvent::Initialized {
                    id,
                    accountant,
                    closed_as_of,
                    closing_tx_id,
                    chart_id,
                } => {
                    let last_closing = PeriodClosing::new(*closed_as_of, *closing_tx_id);
                    builder = builder
                        .id(*id)
                        .last_closing(last_closing)
                        .chart_id(*chart_id);
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
    pub(super) chart_id: ChartId,
    pub(super) closed_as_of: NaiveDate,
    pub(super) closing_tx_id: Option<CalaTxId>,
    pub(super) accountant: Option<String>,
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
            [LedgerClosingEvent::Initialized {
                id: self.id,
                accountant: self.accountant,
                closed_as_of: self.closed_as_of,
                closing_tx_id: self.closing_tx_id,
                chart_id: self.chart_id,
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
