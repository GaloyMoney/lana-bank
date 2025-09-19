select
    s.id as deposit_account_id,
    s.*

from {{ source("lana", "public_core_deposit_account_events_rollup_view") }} as s
