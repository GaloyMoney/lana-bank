select
    s.id as liquidation_process_id,
    s.*

from {{ source("lana", "public_core_liquidation_process_events_rollup_view") }} as s
