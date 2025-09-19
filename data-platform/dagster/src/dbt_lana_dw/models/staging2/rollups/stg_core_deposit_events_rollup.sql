select
    s.id as deposit_id,
    s.*

from {{ source("lana", "public_core_deposit_events_rollup_view") }} as s
