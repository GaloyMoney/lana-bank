select
    s.id as collateral_id,
    s.*

from {{ source("lana", "public_core_collateral_events_rollup_view") }} as s
