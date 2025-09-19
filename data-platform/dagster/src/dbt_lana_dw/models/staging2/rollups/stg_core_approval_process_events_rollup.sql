select
    s.id as approval_process_id,
    s.*

from {{ source("lana", "public_core_approval_process_events_rollup_view") }} as s
