select
    s.id as custodian_id,
    s.*

from {{ source("lana", "public_core_custodian_events_rollup_view") }} as s
