select
    s.id as document_id,
    s.*

from {{ source("lana", "public_core_document_events_rollup_view") }} as s
