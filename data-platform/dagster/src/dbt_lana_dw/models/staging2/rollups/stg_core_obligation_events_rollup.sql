select
    s.id as obligation_id,
    s.*

from {{ source("lana", "public_core_obligation_events_rollup_view") }} as s
