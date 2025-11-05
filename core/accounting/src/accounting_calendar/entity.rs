use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingCalendarId")]
pub enum AccountingCalendarEvent {
    Initialized { id: AccountingCalendarId },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccountingCalendar {
    pub id: AccountingCalendarId,

    events: EntityEvents<AccountingCalendarEvent>,
}

impl TryFromEvents<AccountingCalendarEvent> for AccountingCalendar {
    fn try_from_events(
        events: EntityEvents<AccountingCalendarEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = AccountingCalendarBuilder::default();

        for event in events.iter_all() {
            match event {
                AccountingCalendarEvent::Initialized { id, .. } => builder = builder.id(*id),
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewAccountingCalendar {
    #[builder(setter(into))]
    pub(super) id: AccountingCalendarId,
}

impl NewAccountingCalendar {
    pub fn builder() -> NewAccountingCalendarBuilder {
        NewAccountingCalendarBuilder::default()
    }
}

impl IntoEvents<AccountingCalendarEvent> for NewAccountingCalendar {
    fn into_events(self) -> EntityEvents<AccountingCalendarEvent> {
        EntityEvents::init(
            self.id,
            [AccountingCalendarEvent::Initialized { id: self.id }],
        )
    }
}
