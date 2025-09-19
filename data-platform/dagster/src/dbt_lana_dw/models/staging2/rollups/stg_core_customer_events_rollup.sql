select
    s.id as customer_id,
    s.*

from {{ source("lana", "public_core_customer_events_rollup_view") }} as s
