select
    s.id as interest_accrual_cycle_id,
    s.*

from {{ source("lana", "public_core_interest_accrual_cycle_events_rollup_view") }} as s
