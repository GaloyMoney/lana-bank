select
    id,
    sequence,
    event_type,
    event,
    recorded_at,
    event as parsed_event

from {{ source("lana", "public_core_chart_events_view") }}
