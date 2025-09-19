select
    s.id as manual_transaction_id,
    s.*

from {{ source("lana", "public_core_manual_transaction_events_rollup_view") }} as s
