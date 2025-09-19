select
    s.id as payment_id,
    s.*

from {{ source("lana", "public_core_payment_events_rollup_view") }} as s
