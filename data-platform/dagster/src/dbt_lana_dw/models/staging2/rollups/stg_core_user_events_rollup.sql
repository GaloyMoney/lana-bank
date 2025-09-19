select
    s.id as user_id,
    s.*

from {{ source("lana", "public_core_user_events_rollup_view") }} as s
