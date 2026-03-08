use async_graphql::{
    SimpleObject,
    connection::{Connection, CursorType, Edge, EmptyFields},
};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

use super::primitives::Json;

#[derive(SimpleObject, Clone)]
pub struct EventTimelineEntry {
    pub sequence: i32,
    pub event_type: String,
    pub recorded_at: Timestamp,
    pub payload: Json,
    pub user_id: Option<AuditSubjectId>,
    pub audit_entry_id: Option<AuditEntryId>,
}

#[derive(Serialize, Deserialize)]
pub struct EventTimelineCursor {
    pub sequence: i32,
}

impl CursorType for EventTimelineCursor {
    type Error = String;

    fn encode_cursor(&self) -> String {
        use base64::{Engine as _, engine::general_purpose};
        let json = serde_json::to_string(&self).expect("could not serialize cursor");
        general_purpose::STANDARD_NO_PAD.encode(json.as_bytes())
    }

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

pub fn events_to_connection<E>(
    events: &es_entity::EntityEvents<E>,
    first: i32,
    after: Option<String>,
) -> async_graphql::Result<
    Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
>
where
    E: es_entity::EsEvent + serde::Serialize,
{
    let after_sequence = after
        .map(|cursor_str| EventTimelineCursor::decode_cursor(&cursor_str))
        .transpose()
        .map_err(|e| async_graphql::Error::new(format!("Invalid cursor: {e}")))?
        .map(|c| c.sequence);

    let all_events: Vec<_> = events.iter_persisted().collect();

    let filtered: Vec<_> = if let Some(after_seq) = after_sequence {
        all_events
            .into_iter()
            .filter(|pe| pe.sequence as i32 > after_seq)
            .collect()
    } else {
        all_events
    };

    let first = first.max(0) as usize;
    let has_next_page = filtered.len() > first;
    let page = &filtered[..filtered.len().min(first)];

    let mut connection =
        Connection::<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>::new(
            false,
            has_next_page,
        );

    for pe in page {
        let sequence = pe.sequence as i32;

        let event_type = pe.event.event_type().to_string();
        let payload = serde_json::to_value(&pe.event)
            .map_err(|e| async_graphql::Error::new(format!("Failed to serialize event: {e}")))?;

        // Extract audit info from event context
        let audit_info = audit::AuditInfo::from_context(&pe.context);

        let entry = EventTimelineEntry {
            sequence,
            event_type,
            recorded_at: pe.recorded_at.into(),
            payload: Json::from(payload),
            user_id: audit_info
                .as_ref()
                .map(|a| AuditSubjectId::from(a.sub.clone())),
            audit_entry_id: audit_info.map(|a| AuditEntryId::from(a.audit_entry_id)),
        };

        connection
            .edges
            .push(Edge::new(EventTimelineCursor { sequence }, entry));
    }

    Ok(connection)
}
