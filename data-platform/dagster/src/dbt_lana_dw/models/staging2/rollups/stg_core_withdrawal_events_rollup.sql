select
    s.id as withdrawal_id,
    s.*

from {{ source("lana", "public_core_withdrawal_events_rollup_view") }} as s
