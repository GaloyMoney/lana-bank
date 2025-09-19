select
    s.id as role_id,
    s.*

from {{ source("lana", "public_core_role_events_rollup_view") }} as s
