select
    s.id as permission_set_id,
    s.*

from {{ source("lana", "public_core_permission_set_events_rollup_view") }} as s
