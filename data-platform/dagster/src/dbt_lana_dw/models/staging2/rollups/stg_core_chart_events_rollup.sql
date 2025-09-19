select
    s.id as chart_id,
    s.*

from {{ source("lana", "public_core_chart_events_rollup_view") }} as s
