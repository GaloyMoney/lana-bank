select
    s.id as policy_id,
    s.*

from {{ source("lana", "public_core_policy_events_rollup_view") }} as s
