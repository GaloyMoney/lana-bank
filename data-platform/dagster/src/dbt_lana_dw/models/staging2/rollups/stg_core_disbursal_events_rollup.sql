select
    s.id as disbursal_id,
    s.*

from {{ source("lana", "public_core_disbursal_events_rollup_view") }} as s
