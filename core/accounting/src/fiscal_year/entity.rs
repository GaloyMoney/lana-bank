use chrono::{Datelike, NaiveDate};
use derive_builder::Builder;
use es_entity::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
//use tracing::instrument;

//use super::error::FiscalYearError;
use crate::primitives::{ChartId, FiscalYearId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "FiscalYearId")]
pub enum FiscalYearEvent {
    Initialized {
        id: FiscalYearId,
        chart_id: ChartId,
        reference: String,
        opened_as_of: chrono::NaiveDate,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct FiscalYear {
    pub id: FiscalYearId,
    pub chart_id: ChartId,
    pub reference: String,
    pub opened_as_of: NaiveDate,

    events: EntityEvents<FiscalYearEvent>,
}

impl FiscalYear {}

impl TryFromEvents<FiscalYearEvent> for FiscalYear {
    fn try_from_events(events: EntityEvents<FiscalYearEvent>) -> Result<Self, EsEntityError> {
        let mut builder = FiscalYearBuilder::default();

        for event in events.iter_all() {
            match event {
                FiscalYearEvent::Initialized {
                    id,
                    chart_id,
                    reference,
                    opened_as_of,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .chart_id(*chart_id)
                        .reference(reference.to_string())
                        .opened_as_of(*opened_as_of)
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewFiscalYear {
    #[builder(setter(into))]
    pub id: FiscalYearId,
    #[builder(setter(into))]
    pub chart_id: ChartId,
    pub opened_as_of: NaiveDate,
}

impl NewFiscalYear {
    pub fn builder() -> NewFiscalYearBuilder {
        NewFiscalYearBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        format!("{}:AC{}", self.chart_id, self.opened_as_of.year())
    }
}

impl IntoEvents<FiscalYearEvent> for NewFiscalYear {
    fn into_events(self) -> EntityEvents<FiscalYearEvent> {
        EntityEvents::init(
            self.id,
            [FiscalYearEvent::Initialized {
                id: self.id,
                chart_id: self.chart_id,
                reference: self.reference(),
                opened_as_of: self.opened_as_of,
            }],
        )
    }
}
