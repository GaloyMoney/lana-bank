use async_graphql::{
    ComplexObject, Context, SimpleObject,
    connection::{Connection, CursorType, Edge, EmptyFields},
};
use serde::{Deserialize, Serialize};

use lana_app::primitives::Subject as DomainSubject;

use crate::primitives::*;

use super::{
    audit::{AuditSubject, System},
    loader::*,
    primitives::Json,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct EventTimelineEntry {
    pub sequence: i32,
    pub event_type: String,
    pub recorded_at: Timestamp,
    pub payload: Json,
    pub audit_entry_id: Option<AuditEntryId>,

    #[graphql(skip)]
    pub subject: Option<DomainSubject>,
}

#[ComplexObject]
impl EventTimelineEntry {
    async fn subject(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<AuditSubject>> {
        let subject = match &self.subject {
            Some(s) => s,
            None => return Ok(None),
        };

        let loader = ctx.data_unchecked::<LanaDataLoader>();

        match subject {
            DomainSubject::User(id) => {
                let user = loader.load_one(*id).await?;
                match user {
                    None => Err("User not found".into()),
                    Some(user) => Ok(Some(AuditSubject::User(user))),
                }
            }
            DomainSubject::System(actor) => {
                Ok(Some(AuditSubject::System(System::from_actor(actor))))
            }
            DomainSubject::Customer(_) => {
                panic!("Whoops - have we gone live yet?");
            }
        }
    }
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

    let mut all_events: Vec<_> = events.iter_persisted().collect();
    all_events.reverse();

    let filtered: Vec<_> = if let Some(after_seq) = after_sequence {
        all_events
            .into_iter()
            .filter(|pe| (pe.sequence as i32) < after_seq)
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

        let subject = audit_info
            .as_ref()
            .and_then(|a| a.sub.parse::<DomainSubject>().ok());

        let entry = EventTimelineEntry {
            sequence,
            event_type,
            recorded_at: pe.recorded_at.into(),
            payload: Json::from(payload),
            audit_entry_id: audit_info.map(|a| AuditEntryId::from(a.audit_entry_id)),
            subject,
        };

        connection
            .edges
            .push(Edge::new(EventTimelineCursor { sequence }, entry));
    }

    Ok(connection)
}
